use display_info::error::DIError;
use enigo::InputError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("I/O error: {0}")]
  IoError(#[from] std::io::Error),

  #[error("Input error: {0}")]
  InputError(#[from] InputError),

  #[error("Window state error: {0}")]
  WindowStateError(String),

  #[error("Window list error: {0}")]
  WindowListError(String),

  #[error("Window title error: {0}")]
  WindowTitleError(String),

  #[error("Clipboard error: {0}")]
  ClipboardError(#[from] arboard::Error),

  #[error("DIError: {0}")]
  DIError(#[from] DIError),

  #[error("Notify Error: {0}")]
  NotifyError(#[from] notify::Error),

  #[error("No watch paths configured")]
  NoWatchPaths(),

  #[error("File watcher stopped unexpectedly")]
  FileWatcherStopped,

  #[error("Callback error: {0}")]
  CallbackError(DynError),

  #[error("Failed to send trigger event (channel full or closed)")]
  ChannelSend,
}

impl From<DynError> for Error {
  fn from(err: DynError) -> Self {
    Error::CallbackError(err)
  }
}

type DynError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
