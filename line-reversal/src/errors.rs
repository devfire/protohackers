use thiserror::Error;

/// LRCPError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum LRCPError {
    /// Nom parser was unable to parse the in-bound message
    #[error("Unable to parse message")]
    ParseFailure,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

