// integration test for login 


use std::net::TcpStream; 
use std::process::{Child, Command}; 
use std::thread::sleep;
use std::time::{Duration, Instant};

use login::models::simulator_login_protocol::{SimulatorLoginProtocol};

// port and address for the test server
const PORT: u16 = 8000;
const URL: &'static str= "http://127.0.0.1"; 

struct Reap(Child); 
impl Drop for Reap{
    fn drop (&mut self){
        self.0.kill().expect("process already died");
    } 
}

// sets up python xmlrpc server for testing
fn setup() -> Result <Reap, ()> {
    // logs when server started 
    let start = Instant::now(); 
    // runs the python command to start the test server
    let mut child = match Command::new("python3").arg("tests/test_server/test_server.py").spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "could not start test server, ignoring test({})",
                e 
            );
            return Err(());
        }   
    };

    // logs how many tries it took to connect to server
    let mut iteration = 0; 
    // attempts to connect to python server
    loop {
        match child.try_wait().unwrap(){
            None => {} 
            Some(status) => panic!("python process died {}", status),
        }
        match TcpStream::connect(("localhost", PORT)) {
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
        iteration += 1;
    }
}

// runs the tests
fn run_tests(){
    // this is the test login command
    // will make this better to test logging in
    let test_login: SimulatorLoginProtocol = SimulatorLoginProtocol{
        first: "1".to_string(),
        last: "2".to_string(),
        passwd: "1".to_string(),
        start:"1".to_string(),
        channel:"1".to_string(),
        version:"1".to_string(),
        platform:"1".to_string(),
        platform_string:"1".to_string(),
        platform_version:"1".to_string(),
        mac:"1".to_string(),
        id0:"1".to_string(),
        agree_to_tos: false,
        read_critical: false,
        viewer_digest: "1".to_string(),
        address_size:"1".to_string(),
        extended_errors:"1".to_string(),
        last_exec_event:1,
        last_exec_duration:"1".to_string(),
        skipoptional:false,
        options:"1".to_string(),
    };


    // creates the url string to connect to serve
    // TODO: determine if this is a good way to do it 
    let mut url_string = "".to_owned(); 
    url_string.push_str(URL);
    url_string.push_str(":");
    url_string.push_str(&PORT.to_string());

    println!("url string {}", url_string);


    let req = xmlrpc::Request::new("Login").arg(1).arg(2);
    debug_request_xml(req.clone());
    let login = req.call_url(&url_string).unwrap();
    debug_response_xml(login.clone());
    assert_eq!(login.as_i64(), Some(1 + 2));
}

// prints out xml of request for debugging 
fn debug_request_xml(xml: xmlrpc::Request){
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml request: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug request xml {}", e),
    };
}

// prints out xml of response for debugging 
fn debug_response_xml(xml: xmlrpc::Value){
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml response: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug response xml {}", e),
    };
}

#[test]
fn main(){
    let mut reaper = match setup() {
        Ok(reap) => reap, 
        Err(()) => return,
    };
    
    match reaper.0.try_wait().unwrap() {
        None => {} 
        Some(status)=> {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
    run_tests(); 
    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
}
