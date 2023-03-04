use thiserror::Error;

/// SpeedDaemonError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum SpeedDaemonError {
    /// When the client does something that this protocol specification declares "an error",
    /// the server must send the client an appropriate Error message,
    /// and immediately disconnect that client.
    #[error("Invalid message type")]
    InvalidMessage,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
