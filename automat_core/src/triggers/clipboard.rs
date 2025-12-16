use crate::triggers::context::TriggerContext;
use crate::{callback, get_clipboard_text, pair_api, send_err, Result, Trigger, TriggerRuntime};
use async_trait::async_trait;
use std::fmt::Debug;
use std::time::Duration;
use derivative::Derivative;
use tokio::time::sleep;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ClipboardTrigger {
  last_content: String,
  poll_interval: Duration,
  #[derivative(Debug = "ignore")]
  callback: ClipboardCallback<TriggerContext<ClipboardEvent>>,
}

#[derive(Debug, Clone)]
// New value of the clipboard.
pub struct ClipboardEvent(String);

impl ClipboardEvent {
  pub fn new(content: String) -> Self {
    Self(content)
  }

  pub fn content(&self) -> &str {
    &self.0
  }
}

callback!(ClipboardCallback<T>);

impl ClipboardTrigger {
  pair_api! {
    assoc
      new(f: F)
        callback(TriggerContext<ClipboardEvent>)
        async => Self::with_interval(f, Duration::from_millis(250));
        blocking => Self::with_interval_blocking(f, Duration::from_millis(250));
  }

  pair_api! {
    assoc
      with_interval(f: F, poll_interval: Duration)
        callback(TriggerContext<ClipboardEvent>)
        async => Self { callback: new_clipboard_callback(f), last_content: String::new(), poll_interval };
        blocking => Self { callback: new_clipboard_callback_blocking(f), last_content: String::new(), poll_interval };
  }
}

#[async_trait]
impl Trigger for ClipboardTrigger {
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
    self.last_content = get_clipboard_text()?;

    loop {
      if rt.shutdown.is_cancelled() {
        break;
      }
      let current_content = get_clipboard_text()?;

      if current_content != self.last_content {
        self.last_content = current_content.clone();
        let event = ClipboardEvent(current_content);
        let context = TriggerContext::new(event, rt.tx.clone());

        send_err!(
          (self.callback)(context).await,
          "ClipboardTrigger",
          &rt.tx,
          break
        );
      }

      tokio::select! {
        _ = rt.shutdown.cancelled() => break,
        _ = sleep(self.poll_interval) => {}
      }
    }

    Ok(())
  }

  fn name(&self) -> String {
    "ClipboardTrigger".to_string()
  }
}
