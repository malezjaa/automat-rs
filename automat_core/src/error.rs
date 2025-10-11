use display_info::error::DIError;
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

    #[error("Window list error: {0}")]
    WindowListError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),

    #[error("DIError: {0}")]
    DIError(#[from] DIError),

    #[error("Notify Error: {0}")]
    NotifyError(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
