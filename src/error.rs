use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("No source files found")]
    NoSourceFiles,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
