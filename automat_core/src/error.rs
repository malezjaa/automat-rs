use enigo::InputError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("Input error: {0}")]
    InputError(#[from] InputError),

    #[error("Window state error: {0}")]
    WindowStateError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
