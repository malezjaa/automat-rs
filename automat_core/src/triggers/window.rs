use crate::triggers::context::send_error;
use crate::{callback, pair_api, Result, Trigger, TriggerRuntime, Window};
use async_trait::async_trait;

use std::future::Future;

callback!(WindowChangeCallback<T>);

/// WindowTrigger trigger when the current window changes.
pub struct WindowTrigger {
  last_window: Option<Window>,
  callback: WindowChangeCallback<Window>,
}

impl WindowTrigger {
  pair_api! {
    assoc
      new<F, Fut>(f: F)
        where {
          F: Fn(Window) -> Fut + Send + Sync + 'static,
          Fut: Future<Output = Result<()>> + Send + 'static,
        }
        => Self { last_window: None, callback: new_window_change_callback(f) };
      new_blocking<F>(f: F)
        where {
          F: Fn(Window) -> Result<()> + Send + Sync + 'static,
        }
        => Self { last_window: None, callback: new_window_change_callback_blocking(f) };
  }
}

#[async_trait]
impl Trigger for WindowTrigger {
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
    use tokio::time::{interval, Duration};
    let mut ticker = interval(Duration::from_millis(500));

    loop {
      tokio::select! {
        _ = rt.shutdown.cancelled() => break,
        _ = ticker.tick() => {
          if let Some(window) = Window::current() {
            if self.last_window.as_ref() != Some(&window) {
              self.last_window = Some(window.clone());
              if let Err(err) = (self.callback)(window).await {
                if !send_error(&rt.tx, err, "WindowTrigger").await {
                  break;
                }
              }
            }
          }
        }
      }
    }

    Ok(())
  }

  fn name(&self) -> String {
    "WindowTrigger".to_string()
  }
}
