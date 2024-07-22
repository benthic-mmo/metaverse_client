// this is the object definition for the sim server itself.
// this allows for reading input and output

use actix::prelude::*;
use core::fmt;
use log::{error, info, warn};
use regex::Regex;
use std::borrow::Borrow;
use std::fs::File;
use std::io::{Error as IOError, Write};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::sync::Notify;

use super::conf_spec::SimulatorConfig;
use super::region::RegionsConfig;
use super::standalone_spec::StandaloneConfig;

#[derive(Message)]
#[rtype(result = "()")]
pub struct CommandMessage {
    pub command: String,
}
impl CommandMessage {
    pub fn to_string(&self) -> String {
        self.command.to_string()
    }
}

pub struct ExecData {
    pub base_dir: String,
    pub sim_executable: String,
    pub init_command: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct StdoutMessage {
    pub timestamp: String,
    pub component: ServerComponents,
    pub log_content: String,
}

#[derive(PartialEq, Clone)]
pub enum ServerComponents {
    Console(String),
    Shutdown(String),
    Else(String),
}
impl fmt::Display for ServerComponents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerComponents::Console(ref s) => write!(f, "{}", s),
            ServerComponents::Shutdown(ref s) => write!(f, "{}", s),
            ServerComponents::Else(ref s) => write!(f, "{}", s),
        }
    }
}

#[derive(PartialEq)]
pub enum ServerState {
    Starting,
    Started,
    Stopping,
    Stopped,
}
struct Started;
impl Message for Started {
    type Result = ();
}

struct Stopped;
impl Message for Stopped {
    type Result = ();
}
pub struct SimServer {
    pub state: ServerState,
    pub sim_config: SimulatorConfig,
    pub standalone_config: StandaloneConfig,
    pub regions_config: RegionsConfig,
    pub process: Option<Child>,
    pub process_stdin_receiver: Option<mpsc::Receiver<CommandMessage>>,
    pub process_stdin_sender: Option<mpsc::Sender<CommandMessage>>,
    pub notify: Arc<Notify>,
    pub exec_data: ExecData,
}
impl Actor for SimServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actix Server has started");
        self.state = ServerState::Starting;
        let config_path = format!("{}/bin/OpenSim.ini", self.exec_data.base_dir);
        match SimServer::write_to_file(&config_path, self.sim_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        let config_path = format!(
            "{}/bin/config-include/StandaloneCommon.ini",
            self.exec_data.base_dir
        );
        match SimServer::write_to_file(&config_path, self.standalone_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        let config_path = format!("{}/bin/Regions/Regions.ini", self.exec_data.base_dir);
        match SimServer::write_to_file(&config_path, self.regions_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        match Command::new(self.exec_data.init_command.clone())
            .arg(format!(
                "{}/bin/{}",
                self.exec_data.base_dir, self.exec_data.sim_executable
            ))
            .current_dir(format!("{}/bin", self.exec_data.base_dir))
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                self.process = Some(child);
                info!(
                    "Started server at {}/bin/{}",
                    self.exec_data.base_dir, self.exec_data.sim_executable
                );
            }
            Err(e) => {
                error!(
                    "Failed to start server at {}/bin/{} : {}",
                    self.exec_data.base_dir, self.exec_data.sim_executable, e
                );
                return;
            }
        };
        self.handle_stdout(ctx.address());
        self.handle_stdin();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Actor is stopped ")
    }
}

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
        match msg.component {
            ServerComponents::Console(_) => {
                if !(self.state == ServerState::Started)
                    && msg.log_content.contains("Currently selected region is")
                {
                    self.set_state(ServerState::Started, ctx);
                }
            }
            ServerComponents::Shutdown(_) => {
                if self.state != ServerState::Stopped {
                    self.state != ServerState::Stopping;
                }
                self.state = ServerState::Stopping;
                if msg.log_content.contains("complete") {
                    self.set_state(ServerState::Stopped, ctx);
                }
            }
            _ => {}
        }
    }
}

impl Handler<Started> for SimServer {
    type Result = ();
    fn handle(&mut self, _: Started, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Server has started");
    }
}

impl Handler<Stopped> for SimServer {
    type Result = ();
    fn handle(&mut self, _: Stopped, ctx: &mut Context<Self>) -> Self::Result {
        ctx.stop();
        info!("Server has stopped");
    }
}

impl SimServer {
    fn handle_stdout(&mut self, addr: Addr<SimServer>) {
        if let Some(stdout) = self.process.as_mut().unwrap().stdout.take() {
            let get_stdout = async move {
                info!("running stdout capture");
                let reader = tokio::io::BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Some(line) = lines.next_line().await.unwrap() {
                    let re = Regex::new(r"^(\d{2}:\d{2}:\d{2}) - \[(.*?)\]: (.*)$").unwrap();

                    let stdout_message: StdoutMessage;
                    if let Some(captures) = re.captures(&line) {
                        let timestamp = captures.get(1).map_or("", |m| m.as_str());
                        let component = captures.get(2).map_or("", |m| m.as_str());
                        let log_content = captures.get(3).map_or("", |m| m.as_str());
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
                    addr.borrow().clone().do_send(stdout_message);
                }
            };
            Arbiter::current().spawn(get_stdout);
        }
    }

    fn handle_stdin(&mut self) {
        if let Some(stdin) = self.process.as_mut().unwrap().stdin.take() {
            let mut stdin_receiver = self.process_stdin_receiver.take().unwrap();
            let write_stdin = async move {
                info!("running stdin reciever");
                let mut writer = tokio::io::BufWriter::new(stdin);
                while let Some(cmd) = stdin_receiver.recv().await {
                    info!("command received: {}", cmd.command);
                    if let Err(e) = writer.write_all(cmd.command.as_bytes()).await {
                        error!("failed to write command {}: {}", cmd.command, e);
                    }
                    if let Err(e) = writer.write_all(b"\n").await {
                        error!("failed to add newline {}", e);
                    }
                    if let Err(e) = writer.flush().await {
                        error!("failed to flush writer: {}", e);
                    };
                }
            };
            Arbiter::current().spawn(write_stdin);
        }
    }

    fn set_state(&mut self, new_state: ServerState, ctx: &mut Context<Self>) {
        self.state = new_state;
        if let ServerState::Started = self.state {
            self.notify.notify_one();
            ctx.notify(Started);
        }
        if let ServerState::Stopped = self.state {
            self.notify.notify_one();
            ctx.notify(Stopped);
        }
    }

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
