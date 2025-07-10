use thiserror::Error;

#[derive(Error, Debug)]
pub enum WarpError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Command not found: warp-cli is not installed or not in PATH")]
    CommandNotFound,

    #[error("Failed to parse command output: {0}")]
    #[allow(dead_code)] // May be used in future implementations
    ParseError(String),

    #[error("Command timed out: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Registration already exists")]
    #[allow(dead_code)] // May be used in future implementations
    RegistrationExists,

    #[error("No registration found")]
    #[allow(dead_code)] // May be used in future implementations
    NoRegistration,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Disconnection failed: {0}")]
    DisconnectionFailed(String),
}

pub type WarpResult<T> = Result<T, WarpError>;
