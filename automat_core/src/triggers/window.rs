use crate::triggers::context::send_error;
use crate::{callback, pair_api, Result, Trigger, TriggerContext, TriggerRuntime, Window};
use async_trait::async_trait;
use derivative::Derivative;

callback!(WindowChangeCallback<T>);

/// WindowTrigger trigger when the current window changes.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct WindowTrigger {
  last_window: Option<Window>,
  #[derivative(Debug = "ignore")]
  callback: WindowChangeCallback<TriggerContext<Window>>,
}

impl WindowTrigger {
  pair_api! {
    assoc
      new(f: F)
        callback(TriggerContext<Window>)
        async => Self { last_window: None, callback: new_window_change_callback(f) };
        blocking => Self { last_window: None, callback: new_window_change_callback_blocking(f) };
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
              let ctx = TriggerContext::new(window.clone(), rt.tx.clone());

              if let Err(err) = (self.callback)(ctx).await {
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
