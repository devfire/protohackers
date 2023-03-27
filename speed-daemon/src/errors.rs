use anyhow::Error;
use thiserror::Error;

/// SpeedDaemonError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
#[error(transparent)]
pub enum SpeedDaemonError {
    #[error(transparent)]
    Anyhow(#[from] Error),

    #[error("Error message: {0}")]
    CustomError(#[from]String),

    /// Represents all other cases of `std::io::Error`.
    IOError(#[from] std::io::Error),
}
