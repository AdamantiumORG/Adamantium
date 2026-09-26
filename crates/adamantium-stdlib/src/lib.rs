//! Platform-independent building blocks for the official Adamantium standard library.
//!
//! This crate owns host services and their testable contracts. Language bindings can
//! expose these functions without putting operating-system code in the parser or backend.

pub mod asynchronous;
pub mod collections;
pub mod environment;
pub mod filesystem;
pub mod json;
pub mod math;
pub mod network;
pub mod process;
pub mod random;
pub mod testing;
pub mod time;

/// A stable error returned by standard-library operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub operation: &'static str,
    pub message: String,
}

impl Error {
    pub fn new(operation: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            operation,
            message: error.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
