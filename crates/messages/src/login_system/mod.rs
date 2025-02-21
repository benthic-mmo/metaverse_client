//! # Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol)
//!
//! `metaverse_login` is a utility for running login commands against opensimulator or secondlife
//! servers.
//login functions for logging into metaverse servers
pub mod errors;
pub mod login;
pub mod login_response;
pub mod simulator_login_protocol;
