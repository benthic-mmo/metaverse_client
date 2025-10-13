use serde::{Deserialize, Serialize};

use crate::packet::message::UIResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The struct required for constructing a login
pub struct Login {
    /// first name
    pub first: String,
    /// last name
    pub last: String,
    /// password (md5 hashed)
    pub passwd: String,
    /// where the user logs in
    pub start: String,
    /// the name of the viewer
    pub channel: String,
    /// did the user agree to the TOS
    pub agree_to_tos: bool,
    /// did the user read critical announcements
    pub read_critical: bool,
    /// the URL that the user is logging in against
    pub url: String,
}

impl UIResponse {
    /// allow sending the login object between the UI and Core
    pub fn new_login_event(data: Login) -> Self {
        UIResponse::Login(data)
    }
}
