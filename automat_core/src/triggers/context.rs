use crate::{Error, Result};
use tokio::sync::mpsc;

/// Sent between channels to indicate trigger events.
#[derive(Debug)]
pub enum TriggerEvent {
  Error(Error),
  /// This stops the trigger.
  ErrorFatal(Error),
  Stop,
}

#[derive(Debug)]
pub struct TriggerContext<T> {
  pub data: T,
  tx: mpsc::Sender<TriggerEvent>,
}

impl<T> TriggerContext<T> {
  pub fn new(data: T, tx: mpsc::Sender<TriggerEvent>) -> Self {
    Self { data, tx }
  }

  /// Send an error event. Returns Err if the channel is full or closed.
  pub fn error(&self, error: Error) -> Result<()> {
    self.tx.try_send(TriggerEvent::Error(error)).map_err(|e| {
      eprintln!(
        "Warning: Failed to send error event (channel backpressure): {:?}",
        e
      );
      Error::ChannelSend
    })
  }

  /// Send a fatal error event. Returns Err if the channel is full or closed.
  pub fn error_fatal(&self, error: Error) -> Result<()> {
    self
      .tx
      .try_send(TriggerEvent::ErrorFatal(error))
      .map_err(|e| {
        eprintln!(
          "Warning: Failed to send fatal error event (channel backpressure): {:?}",
          e
        );
        Error::ChannelSend
      })
  }

  /// Sends a stop signal to the trigger. Returns Err if the channel is full or closed.
  pub fn stop(&self) -> Result<()> {
    self.tx.try_send(TriggerEvent::Stop).map_err(|e| {
      eprintln!(
        "Warning: Failed to send stop event (channel backpressure): {:?}",
        e
      );
      Error::ChannelSend
    })
  }
}

pub type TriggerChannel = (mpsc::Sender<TriggerEvent>, mpsc::Receiver<TriggerEvent>);

pub async fn send_error(tx: &mpsc::Sender<TriggerEvent>, err: Error, trigger_name: &str) -> bool {
  if tx.send(TriggerEvent::Error(err)).await.is_err() {
    eprintln!(
      "Warning: {} event channel closed, stopping trigger",
      trigger_name
    );
    return false;
  }
  true
}

#[macro_export]
macro_rules! send_err {
  (
        $expr:expr,
        $name:expr,
        $tx:expr,
        $ret:expr
    ) => {
    if let Err(err) = ($expr) {
      if !crate::send_error($tx, err, $name).await {
        $ret
      }
    }
  };
}
