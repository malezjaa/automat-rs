use crate::triggers::context::send_error;
use crate::{
  callback, pair_api, Result, Trigger, TriggerContext,
  TriggerRuntime,
};
use async_trait::async_trait;
use derivative::Derivative;
use std::time::Duration;

callback!(IntervalCallback<T>);

/// IntervalTrigger yields at a fixed time period.
///
/// Use this trigger to schedule recurring actions with a constant interval.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct IntervalTrigger {
  interval: Duration,
  #[derivative(Debug = "ignore")]
  callback: IntervalCallback<TriggerContext<Duration>>,
}

impl IntervalTrigger {
  pair_api! {
    assoc
      new(interval: Duration, f: F)
        callback(TriggerContext<Duration>)
        async => Self { interval, callback: new_interval_callback(f) };
        blocking => Self { interval, callback: new_interval_callback_blocking(f) };
  }

  #[inline(always)]
  pub fn interval(&self) -> Duration {
    self.interval
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
          let ctx = TriggerContext::new(self.interval(), rt.tx.clone());

          if let Err(err) = (self.callback)(ctx).await {
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
