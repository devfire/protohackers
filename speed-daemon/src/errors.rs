use thiserror::Error;

/// SpeedDaemonError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum SpeedDaemonError {
    /// Nom parser was unable to parse the in-bound message
    #[error("Unable to parse message")]
    ParseFailure,

    /// Duplicate client
    #[error("Duplicate camera detected")]
    DuplicateCamera,

    /// Duplicate client
    #[error("Duplicate dispatcher detected")]
    DuplicateDispatcher,

    #[error("Client disconnected")]
    DisconnectedClient,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
