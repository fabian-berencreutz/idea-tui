use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum IdeaError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Config error: {0}")]
    Config(#[from] confy::ConfyError),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Project error: {0}")]
    Project(String),

    #[error("Clone failed for: {0}")]
    CloneFailed(String),

    #[error("Process spawn error: {0}")]
    Spawn(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, IdeaError>;
