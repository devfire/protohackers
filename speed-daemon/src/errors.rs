use thiserror::Error;

/// SpeedDaemonError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum SpeedDaemonError {
    /// Received an invalid message type
    #[error("Invalid message type")]
    InvalidMessage,

    /// Nom parser was unable to parse the in-bound message
    #[error("Unable to parse message")]
    ParseFailure,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
