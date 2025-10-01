use crate::{Action, Result, Trigger, async_callback, impl_display_debug};
use async_trait::async_trait;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::interval;

async_callback!(IntervalCallback<T>);

/// IntervalTrigger yields at a fixed time period.
///
/// Use this trigger to schedule recurring actions with a constant interval.
pub struct IntervalTrigger {
    interval: Duration,
    callback: IntervalCallback<Duration>,
}

impl IntervalTrigger {
    pub fn new<F, Fut>(interval: Duration, f: F) -> Self
    where
        F: Fn(Duration) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            interval,
            callback: new_interval_callback(f),
        }
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
impl Action for IntervalTrigger {
    async fn run(&self) -> Result<()> {
        self.call().await
    }
}

#[async_trait]
impl Trigger for IntervalTrigger {
    async fn start(&mut self) -> Result<()> {
        let mut ticker = interval(self.interval());
        ticker.tick().await; // Skip the immediate first tick

        loop {
            ticker.tick().await;
            self.run().await?;
        }
    }

    fn name(&self) -> String {
        format!(
            "IntervalTrigger with period of {} milliseconds",
            self.interval.as_millis()
        )
    }
}

impl_display_debug!(IntervalTrigger, |self, f| write!(f, "{}", self.name()));
