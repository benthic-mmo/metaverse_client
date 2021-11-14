use crate::models::errors::{LoginError, Reason};
use crate::models::session_data::Session;

pub fn new_session(login_response: xmlrpc::Value) -> Result<Session, LoginError> {
    println!("{:?}", login_response);
    match login_response["login"].as_str().unwrap() {
        // if login is true then login succeeded
        "true" => Ok(login_response.into()),
        // if login is false then login failed
        "false" => match login_response.get("reason") {
            // if no reason is sent, something strange happened
            None => Err(LoginError {
                reason: Reason::Unknown,
                message: format!("{:?}", login_response),
            }),
            Some(x) => match x.as_str().unwrap() {
                "key" => Err(LoginError {
                    reason: Reason::Key,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
                "presence" => Err(LoginError {
                    reason: Reason::Presence,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
                _ => Err(LoginError {
                    reason: Reason::Unknown,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
            },
        },
        &_ => Err(LoginError {
            reason: Reason::Unknown,
            message: "".to_string(),
        }),
    }
}
