use crate::triggers::context::send_error;
use crate::{Result, Trigger, TriggerEvent, Window, callback};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

callback!(WindowChangeCallback<T>);

/// WindowTrigger trigger when the current window changes.
pub struct WindowTrigger {
  last_window: Option<Window>,
  callback: WindowChangeCallback<Window>,
}

impl WindowTrigger {
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(Window) -> Result<()> + Send + Sync + 'static,
  {
    Self {
      last_window: None,
      callback: new_window_change_callback(f),
    }
  }
}

#[async_trait]
impl Trigger for WindowTrigger {
  async fn start(&mut self, tx: Sender<TriggerEvent>) {
    use tokio::time::{Duration, interval};
    let mut ticker = interval(Duration::from_millis(500));

    loop {
      ticker.tick().await;
      if let Some(window) = Window::current() {
        if self.last_window.as_ref() != Some(&window) {
          self.last_window = Some(window.clone());
          if let Err(err) = (self.callback)(window) {
            if !send_error(&tx, err, "WindowTrigger").await {
              break;
            }
          }
        }
      }
    }
  }

  fn name(&self) -> String {
    "WindowTrigger".to_string()
  }
}
