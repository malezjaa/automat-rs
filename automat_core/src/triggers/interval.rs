use crate::triggers::context::send_error;
use crate::{callback, impl_display_debug, pair_api, ActionAsync, Result, Trigger, TriggerRuntime};
use async_trait::async_trait;
use std::time::Duration;

callback!(IntervalCallback<T>);

/// IntervalTrigger yields at a fixed time period.
///
/// Use this trigger to schedule recurring actions with a constant interval.
pub struct IntervalTrigger {
  interval: Duration,
  callback: IntervalCallback<Duration>,
}

impl IntervalTrigger {
  pair_api! {
    assoc
      new(interval: Duration, f: F)
        callback(Duration)
        async => Self { interval, callback: new_interval_callback(f) };
        blocking => Self { interval, callback: new_interval_callback_blocking(f) };
  }

  #[inline(always)]
  pub fn interval(&self) -> Duration {
    self.interval
  }

  pub async fn call(&self) -> Result<()> {
    (self.callback)(self.interval).await
  }
}

#[async_trait]
/// Action just calls provided callback.
impl ActionAsync for IntervalTrigger {
  async fn run_async(&self) -> Result<()> {
    self.call().await
  }
}

#[async_trait]
impl Trigger for IntervalTrigger {
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
    use tokio::time::interval;
    let mut ticker = interval(self.interval());
    ticker.tick().await; // The first tick completes immediately

    loop {
      tokio::select! {
        _ = rt.shutdown.cancelled() => break,
        _ = ticker.tick() => {
          if let Err(err) = self.call().await {
            if !send_error(&rt.tx, err, "IntervalTrigger").await {
              break;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn name(&self) -> String {
    format!(
      "IntervalTrigger with period of {} milliseconds",
      self.interval.as_millis()
    )
  }
}

impl_display_debug!(IntervalTrigger, |self, f| write!(f, "{}", self.name()));
