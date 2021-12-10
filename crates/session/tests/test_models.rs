use metaverse_session::models::session_data::*;
use metaverse_session::session::connect;

use std::net::UdpSocket;
use std::panic;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};
use uuid::Uuid;

const UDP_PORT: u16 = 20001;

struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        self.0.kill().expect("process already died");
    }
}

pub fn default_session() -> Session {
    Session {
        first_name: Some("first".to_string()),
        last_name: Some("last".to_string()),
        sim_ip: Some("127.0.0.1".to_string()),
        sim_port: Some(UDP_PORT),
        circuit_code: Some(697482820),
        agent_id: Some(Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap()),
        session_id: Some(Uuid::parse_str("6ac2e761-f490-4122-bf6c-7ad8fbb17002").unwrap()),
        ..Session::default()
    }
}

#[test]
fn establish_dummy_conn() {
    let mut reaper = match setup_udp() {
        Ok(reap) => reap,
        Err(_string) => return,
    };
    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
    tokio_test::block_on(connect(default_session())).unwrap();
}

/// sets up python xmlrpc server for testing
fn setup_udp() -> Result<Reap, String> {
    // logs when server started
    let start = Instant::now();
    // runs the python command to start the test server
    let mut child = match Command::new("python3")
        .arg("tests/test_udp/test_udp.py")
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("could not start test server, ignoring test({})", e);
            return Err("Could not start test server".to_string());
        }
    };

    // logs how many tries it took to connect to server
    // attempts to connect to python server
    for iteration in 0.. {
        match child.try_wait().unwrap() {
            None => {}
            Some(status) => panic!("python process died {}", status),
        }
        match UdpSocket::bind(("localhost", UDP_PORT)) {
            Ok(_) => {
                println!(
                    "connected to server after {:?} (iteration{})",
                    Instant::now() - start,
                    iteration
                );
                return Ok(Reap(child));
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(50));
    }
    return Ok(Reap(child));
}
