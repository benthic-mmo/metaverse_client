// this is the object definition for the sim server itself.
// this allows for reading input and output
//
use actix::prelude::*;
use log::{error, info, warn};
use std::fs::File;
use std::io::{BufRead, Error as IOError, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::sync::mpsc;
use std::process::Child;

use super::conf_spec::SimulatorConfig;
use super::region::RegionsConfig;
use super::standalone_spec::StandaloneConfig;

#[derive(Message)]
#[rtype(result = "()")]
pub struct CommandMessage {
    command: String,
}
impl CommandMessage {
    pub fn to_string(&self) -> String {
        self.command.to_string()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartServer {
    pub base_dir: String,
    pub sim_executable: String,
    pub init_command: String,
}

// TODO make this better with new fields
pub struct StdoutMessage(pub String);

pub struct SimServer {
    pub sim_config: SimulatorConfig,
    pub standalone_config: StandaloneConfig,
    pub regions_config: RegionsConfig,
    pub process: Option<Child>,
    pub process_stdout_sender: Option<mpsc::Sender<StdoutMessage>>,
}
impl Actor for SimServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Sim Server has started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Actor is stopped ")
    }
}
impl Handler<CommandMessage> for SimServer {
    type Result = ();

    fn handle(&mut self, msg: CommandMessage, _ctx: &mut Self::Context) -> Self::Result {
        info!("Received message {}", msg.to_string());
    }
}
impl Handler<StartServer> for SimServer {
    type Result = ();

    fn handle(&mut self, msg: StartServer, _ctx: &mut Self::Context) -> Self::Result {
        let config_path = format!("{}/bin/OpenSim.ini", msg.base_dir);
        match SimServer::write_to_file(&config_path, self.sim_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        let config_path = format!("{}/bin/config-include/StandaloneCommon.ini", msg.base_dir);
        match SimServer::write_to_file(&config_path, self.standalone_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        let config_path = format!("{}/bin/Regions/Regions.ini", msg.base_dir);
        match SimServer::write_to_file(&config_path, self.regions_config.to_string()) {
            Ok(()) => info!("Wrote {} to file", &config_path),
            Err(e) => error!("Failed to write {}: {}", &config_path, e),
        }

        match Command::new(msg.init_command)
            .arg(format!("{}/bin/{}", msg.base_dir, msg.sim_executable))
            .current_dir(format!("{}/bin", msg.base_dir))
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                self.process = Some(child);
                info!(
                "Started server at {}/bin/{}",
                msg.base_dir, msg.sim_executable);
            },
            Err(e) => {
                error!(
                    "Failed to start server at {}/bin/{} : {}",
                    msg.base_dir, msg.sim_executable, e
                );
                return()
            }
        };
        // set up stdout capture 
        if let Some(stdout) = self.process.as_mut().unwrap().stdout.take(){
            let stdout_sender = self.process_stdout_sender.clone().unwrap();
            let get_stdout = async move {
                info!("running stdout capture");
                let reader = std::io::BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Err(_) = stdout_sender.send(StdoutMessage(line)).await {
                                break;
                            }
                        }
                    }
            };
            Arbiter::current().spawn(get_stdout);
        }
    }
}

impl SimServer {
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
