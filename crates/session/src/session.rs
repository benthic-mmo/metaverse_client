use crate::models::session_data::Session;

pub fn new_session(login_response: xmlrpc::Value) -> Option<Session> {
    if login_response.get("reason") != None {
        return None; //create an error type. failed to create a session
    }
    Some(login_response.into())
}
