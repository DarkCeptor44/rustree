use std::path::PathBuf;

/// The error type
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The path is not a directory
    #[error("path is not a directory: {0}")]
    NotDir(PathBuf),
}
