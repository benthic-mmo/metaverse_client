use actix::prelude::*;
use core::fmt;
use std::sync::{Arc, Mutex};
use tokio::process::Child;
use tokio::sync::{mpsc, Notify};

use super::conf_spec::SimulatorConfig;
use super::region::RegionsConfig;
use super::standalone_spec::StandaloneConfig;

// this is the object definition for the sim server itself
// this contains the spec for the server object, and all of its message structs and related enums.

#[derive(Message)]
#[rtype(result = "()")]
pub struct CommandMessage {
    pub command: String,
}
impl fmt::Display for CommandMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

pub struct ExecData {
    pub base_dir: String,
    pub executable: String,
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

#[derive(PartialEq, Clone)]
pub enum ServerState {
    Starting,
    Running,
    Stopping,
    Stopped,
}
impl fmt::Display for ServerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerState::Starting => write!(f, "Starting"),
            ServerState::Running => write!(f, "Running"),
            ServerState::Stopping => write!(f, "Stopping"),
            ServerState::Stopped => write!(f, "Stopped"),
        }
    }
}

pub struct SimServer {
    pub state: Arc<Mutex<ServerState>>,
    pub sim_config: SimulatorConfig,
    pub standalone_config: StandaloneConfig,
    pub regions_config: RegionsConfig,
    pub process: Option<Child>,
    pub process_stdin_receiver: Option<mpsc::Receiver<CommandMessage>>,
    pub process_stdin_sender: Option<mpsc::Sender<CommandMessage>>,
    pub process_stdout_sender: Option<mpsc::Sender<StdoutMessage>>,
    pub notify: Arc<Notify>,
    pub exec_data: ExecData,
}
