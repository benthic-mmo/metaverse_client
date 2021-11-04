Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol) 

# Getting Started 
## Using Library 
### [docs](https://docs.rs/metaverse_login/0.0.0/metaverse_login/) 
### [crate](https://crates.io/crates/metaverse_login)

add 
```
metaverse_login = "0.0.0"
```
to your cargo.toml 

## Running Tests 
to run the tests simply run cargo test 
To run tests against live osgrid credentials, 
```
cp .creds.example .creds.toml 
```
edit .creds.toml to include your username, password and login location.
