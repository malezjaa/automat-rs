use crate::Error;
use tokio::sync::mpsc;

/// Sent between channels to indicate trigger events.
pub enum TriggerEvent {
  Error(Error),
  /// This stops the trigger.
  ErrorFatal(Error),
  Stop,
}

pub struct TriggerContext<T> {
  pub data: T,
  tx: mpsc::Sender<TriggerEvent>,
}

impl<T> TriggerContext<T> {
  pub fn new(data: T, tx: mpsc::Sender<TriggerEvent>) -> Self {
    Self { data, tx }
  }

  pub fn error(&self, error: Error) {
    let _ = self.tx.try_send(TriggerEvent::Error(error));
  }

  pub fn error_fatal(&self, error: Error) {
    let _ = self.tx.try_send(TriggerEvent::ErrorFatal(error));
  }

  /// Sends a stop signal to the trigger.
  pub fn stop(&self) {
    let _ = self.tx.try_send(TriggerEvent::Stop);
  }
}

pub type TriggerChannel = (mpsc::Sender<TriggerEvent>, mpsc::Receiver<TriggerEvent>);
