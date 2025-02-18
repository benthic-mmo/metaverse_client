use crate::models::server::*;
use actix::prelude::*;
use log::{error, info, warn};
use regex::Regex;
use reqwest::Url;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Cursor;
use std::io::{Error as IOError, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs::File as tokioFile;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

// This is the Actor for the SimServer.
// this contains all information for
// - starting the sim server
// - reading its stdout
// - writing to stdin
// - setting server state

impl Actor for SimServer {
    type Context = Context<Self>;

    /// Started is called when the Actix Server has started.
    ///
    /// this functions as the main thread where all other processes are spawned.
    ///
    /// this won't run properly, as it needs imports, and to be run within an actix runtime.
    /// However, this is the gits of what you need to do in order to run
    ///
    ///# Examples
    ///```
    ///let url = "http://opensimulator.org/dist/OpenSim-LastAutoBuild.zip";
    ///let sim_archive = "sims/sim.zip";
    ///let sim_path = "sims/opensim";
    ///let sim_executable = "OpenSim.exe";
    ///
    ///let notify = Arc::new(Notify::new());
    ///let state = Arc::new(Mutex::new(ServerState::Starting))
    ///let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);
    ///
    ///let base_dir: String;
    ///
    ///if let Ok(canonical_path) = fs::canonicalize(sim_path) {
    ///     base_dir = canonical_path
    ///                 .into_os_string()
    ///                 .into_string()
    ///                 .unwrap()
    ///                 .to_string();
    ///} else {
    ///     return()
    ///}
    ///
    ///
    ///SimServer{
    ///     state: Arc::clone(&state)
    ///     standalone_config: create_default_standalone_config(),
    ///     regions_config: create_default_regions_config(),
    ///     process: None,
    ///     process_stdout_sender: None, //this is for subscribing to stdout from process
    ///     process_stdin_sender:Some(stdin_sender),
    ///     process_stdin_receiver:Some(stdin_sender)
    ///     notify: Arc::clone(&notify),
    ///     exec_data: ExecData{
    ///         base_dir,
    ///         executable,
    ///         init_command: "mono".to_string(),
    ///     },
    ///}.start();
    ///
    ///notify.notified().await;
    ///
    ///if *state.lock().unwrap() == ServerState::Running{
    ///     sim_server.do_send(CommmandMessage{
    ///         command: "create user default user password email@email.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string
    ///     });
    ///     sim_server.do_send(CommandMessage{
    ///         command:"quit".to_string(),
    ///     });
    ///} else {
    ///     return("server failed to start")
    ///}
    ///
    ///notify.notified().await;
    ///````
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actix Server has started");

        // write the OpenSim.ini to the correct file
        // this is the configuration for the main OpenSim instance
        // TODO: make this generic
        let config_path = format!("{}/bin/OpenSim.ini", self.exec_data.base_dir);
        match SimServer::write_to_file(&config_path, self.sim_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        // write the StandaloneCommon.ini to the corect file
        // this is the configuration for standalone grids.
        // currently the crate only supports standalone grids.
        // TODO: make this generic
        let config_path = format!(
            "{}/bin/config-include/StandaloneCommon.ini",
            self.exec_data.base_dir
        );
        match SimServer::write_to_file(&config_path, self.standalone_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        // write the Regions.ini to the correct file
        // this is the regions configuration, where the region information is kept.
        let config_path = format!("{}/bin/Regions/Regions.ini", self.exec_data.base_dir);
        match SimServer::write_to_file(&config_path, self.regions_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        // spawn a new mono process to start the simulator
        match Command::new(self.exec_data.init_command.clone())
            .arg(format!(
                "{}/bin/{}",
                self.exec_data.base_dir, self.exec_data.executable
            ))
            .current_dir(format!("{}/bin", self.exec_data.base_dir))
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                // store the process in the actor's struct
                self.process = Some(child);

                // set state to starting
                self.set_state(ServerState::Starting, ctx);
                info!(
                    "Started server at {}/bin/{}",
                    self.exec_data.base_dir, self.exec_data.executable
                );
            }
            Err(e) => {
                error!(
                    "Failed to start server at {}/bin/{} : {}",
                    self.exec_data.base_dir, self.exec_data.executable, e
                );
                return;
            }
        };
        // spawn a thread to read stdout of the mono process
        self.handle_stdout(ctx.address());

        // spawn a thread to attach to the stdin of the mono process
        self.handle_stdin();
    }

    // set state to stopped, and send a quit command to the stdin handler
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.set_state(ServerState::Stopped, ctx);
        let process_stdin_sender = self.process_stdin_sender.clone().unwrap();
        actix::spawn(async move {
            process_stdin_sender
                .send(CommandMessage {
                    command: "quit".to_string(),
                })
                .await
                .unwrap();
        });
        info!("Actor is stopped ")
    }
}

// This is the handler for CommandMessage for the SimServer.
// when it receives messages, it sends the CommandMessage to stdin
impl Handler<CommandMessage> for SimServer {
    type Result = ();

    fn handle(&mut self, msg: CommandMessage, _ctx: &mut Self::Context) -> Self::Result {
        let process_stdin_sender = self.process_stdin_sender.clone().unwrap();
        actix::spawn(async move {
            info!("sent command: {}", msg.command);
            process_stdin_sender.send(msg).await.unwrap();
        });
    }
}

// this is the handler for receiving stdout messages
// every captured stdout message get sent as a message back to the actor
// and processed in this function.
impl Handler<StdoutMessage> for SimServer {
    type Result = ();

    fn handle(&mut self, msg: StdoutMessage, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "[{}]: {}",
            msg.component.to_string(),
            msg.log_content.to_string()
        );
        if msg.log_content.contains("Fatal error") {
            self.set_state(ServerState::Stopped, ctx);
        }

        // each message has a component type, which can be set and used to respond to events
        // TODO: use full list of components
        //
        let server_state = self.get_state();
        match msg.component {
            // set the server state to Running once the stdout contains "Currently selected
            // region is"
            // this is the output that the server gives once fully initialized.
            // TODO: make this generic
            ServerComponents::Console(_) => {
                if !(server_state == ServerState::Running)
                    && msg.log_content.contains("Currently selected region is")
                {
                    self.set_state(ServerState::Running, ctx);
                }
            }
            // if the server sends a Shutdown message, set the state to stopping,
            // and if that shutdown message contains the word complete, then the server has
            // stoppped.
            ServerComponents::Shutdown(_) => {
                if server_state != ServerState::Stopped && server_state != ServerState::Stopping {
                    self.set_state(ServerState::Stopping, ctx);
                }
                if msg.log_content.contains("complete") {
                    self.set_state(ServerState::Stopped, ctx);
                }
            }
            _ => {}
        }
    }
}

impl SimServer {
    // this is the function for handling the stdout.
    // it creates an async thread that runs  until the server's state is set to Stopped.
    fn handle_stdout(&mut self, addr: Addr<SimServer>) {
        if let Some(stdout) = self.process.as_mut().unwrap().stdout.take() {
            let state_clone = Arc::clone(&self.state);
            // stdout_sener is optional. it allows for proceses to subscribe to the stdout
            let stdout_sender = self.process_stdout_sender.clone();

            // this is the thread logic
            let get_stdout = async move {
                info!("running stdout capture");
                let reader = tokio::io::BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Some(line) = lines.next_line().await.unwrap() {
                    // if the state of the server is stopped, break the loop, which exits the
                    // thread
                    if *state_clone.lock().unwrap() == ServerState::Stopped {
                        info!("server has stopped, exiting stdout capture");
                        break;
                    }

                    // this regex takes data in the format output by the opensim server
                    // for example, this could be one of the output logs by the mono process
                    // 12:34:56 - [SCENE]: Initializing script instances in default
                    // the regex splits, and adds each part to the component type for use in other
                    // places.
                    // TODO: make this regex more generic
                    let re = Regex::new(r"^(\d{2}:\d{2}:\d{2}) - \[(.*?)\]: (.*)$").unwrap();
                    let stdout_message: StdoutMessage;
                    if let Some(captures) = re.captures(&line) {
                        let timestamp = captures.get(1).map_or("", |m| m.as_str());
                        let component = captures.get(2).map_or("", |m| m.as_str());
                        let log_content = captures.get(3).map_or("", |m| m.as_str());
                        // this is where the component gets assigned to its type enum. this makes
                        // it easy to identify where the log is coming from within the process.
                        // currently the only types it can be are Shutdown, Console and Else. All
                        // messages not from shutdown or console are stored in else.
                        // TODO: create exhaustive list of enums
                        let component_enum = match component {
                            "SHUTDOWN" => ServerComponents::Shutdown("SHUTDOWN".to_string()),
                            _ => ServerComponents::Else(component.to_string()),
                        };
                        stdout_message = StdoutMessage {
                            timestamp: timestamp.to_string(),
                            component: component_enum,
                            log_content: log_content.to_string(),
                        };
                    } else {
                        stdout_message = StdoutMessage {
                            timestamp: "to be dealt with later".to_string(),
                            component: ServerComponents::Console("Console".to_string()),
                            log_content: line,
                        };
                    }
                    // allow for process to subscribe to stdout, and receive message structs
                    // since sender is optional, this does nothing if process_stdout_sender isn't set
                    if let Some(sender) = &stdout_sender {
                        if (sender.send(stdout_message.clone()).await).is_err() {
                            break;
                        }
                    }
                    // do_send sends the stdout message back to the actor, which processes the
                    // data
                    addr.borrow().clone().do_send(stdout_message);
                }
            };
            // spawn the thread
            Arbiter::current().spawn(get_stdout);
        }
    }

    // this functions handles the stdin for the server.
    // when the server receives messages, it sends them to the stdin_sender, which passes them to
    // the stdin_receiver in this thread.
    fn handle_stdin(&mut self) {
        if let Some(stdin) = self.process.as_mut().unwrap().stdin.take() {
            let mut stdin_receiver = self.process_stdin_receiver.take().unwrap();
            let state_clone = Arc::clone(&self.state);
            let write_stdin = async move {
                info!("running stdin reciever");
                let mut writer = tokio::io::BufWriter::new(stdin);
                while let Some(mut cmd) = stdin_receiver.recv().await {
                    // if the server's state is stopped, break the loop and exit the thread
                    if *state_clone.lock().unwrap() == ServerState::Stopped {
                        info!("server has stopped, exiting stdin thread");
                        break;
                    }
                    info!("command received: {}", cmd.command);
                    // append a newline to the command so it goes through
                    cmd.command.push('\n');
                    if let Err(e) = writer.write_all(cmd.command.as_bytes()).await {
                        error!("failed to write command {}: {}", cmd.command, e);
                    }

                    if let Err(e) = writer.flush().await {
                        error!("failed to flush writer: {}", e);
                    };
                }
            };
            //spawn the thread
            Arbiter::current().spawn(write_stdin);
        }
    }

    // sets the state of the server
    fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
        let state_clone = Arc::clone(&self.state);
        {
            let mut state = state_clone.lock().unwrap();
            *state = new_state.clone();
        }
        // notify on start and stop
        if new_state == ServerState::Running || new_state == ServerState::Stopped {
            self.notify.notify_one();
        }
    }

    // get state of the server
    pub fn get_state(&self) -> ServerState {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    // write generated config files to files
    fn write_to_file(path: &String, data: String) -> Result<(), IOError> {
        if !Path::new(&path).exists() {
            let path = Path::new(&path);
            let mut file = File::create(path)?;

            file.write_all(data.as_bytes())?;
            info!("Successfully wrote to {}", path.display());
        } else {
            warn!(
                "{} already exists. Continuing without generating defaults.",
                path
            );
        }
        Ok(())
    }
}

// handle downloading sim
pub async fn download_sim(
    url: &str,
    sim_archive: &str,
    sim_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(sim_archive).exists() {
        let output_dir = Path::new(sim_archive).parent().unwrap();
        std::fs::create_dir_all(output_dir)?;

        info!("downloading sim {:?}", url);
        let url = Url::parse(url)?;
        let response = reqwest::get(url).await?;
        let mut dest = tokioFile::create(sim_archive).await?;
        let content = response.bytes().await?;
        dest.write_all(&content).await?;
        info!("sim downloaded and saved to {}", sim_path);
    } else {
        warn!("archive already exists at {}", sim_archive);
    }

    if !Path::new(sim_path).exists() {
        let mut file = tokioFile::open(sim_archive).await?;
        let mut archive = Vec::new();
        file.read_to_end(&mut archive).await?;

        let target_dir = PathBuf::from(sim_path);

        zip_extract::extract(Cursor::new(archive), &target_dir, true)?;
    } else {
        warn!("sim already exists at {}", sim_path)
    }

    Ok(())
}
