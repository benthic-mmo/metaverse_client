use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Reason {
    Key,
    Presence,
    Unknown,
}

#[derive(Clone)]
pub struct LoginError {
    pub reason: Reason,
    pub message: String,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Reason::Presence => "Presence",
            Reason::Key => "Key",
            Reason::Unknown => "Unknown",
        };
        write!(f, "{}", msg)
    }
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.reason {
            Reason::Presence => {
                "Login failed because you are already logged in. Wait a few minutes and try again."
            }
            Reason::Key => "Username or password incorrect",
            Reason::Unknown => "Unknown error occured.",
        };
        write!(f, "{}", err_msg)
    }
}

impl fmt::Debug for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LoginError {{ reason: {}, message: {} }}",
            self.reason, self.message
        )
    }
}
