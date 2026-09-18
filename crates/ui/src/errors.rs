use benthic_protocol::errors::SessionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialLoadError {
    #[error("failed to read cached user data: {0}")]
    ReadFile(#[from] std::io::Error),

    #[error("error from keyring when loading credentials: {0}")]
    KeyringError(#[from] keyring::Error),
}

#[derive(Debug, Error)]
pub enum ShareDirError {
    #[error("Failed to create share dir:")]
    SessionError(#[from] SessionError),
}

#[derive(Debug, Error)]
pub enum CredentialStoreError {
    #[error("error from keyring when storing credentials: {0}")]
    KeringError(#[from] keyring::Error),

    #[error("error writing to file when storing to credentials: {0}")]
    WriteFile(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CredentialDeleteError {
    #[error("error from keyring when deleting credentials: {0}")]
    KeringError(#[from] keyring::Error),

    #[error("error writing to file when deleting credentials: {0}")]
    WriteFile(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum PacketSendError {
    #[error("Error sending packet")]
    PacketSendError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum NotLoggedIn {
    #[error("You are not logged in")]
    NotLoggedInError(),
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Error sending chat: {0}")]
    ChatLoginError(#[from] NotLoggedIn),

    #[error("Error sending chat: {0}")]
    ChatSendError(#[from] PacketSendError),
}

#[derive(Debug, Error)]
pub enum PortError {
    #[error("Failed to pick port")]
    PortPickerError(),
}
