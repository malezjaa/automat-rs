use crate::window::{WindowIdentifier, get_current_window_identifier};
use crate::{
    Action, ActionAsync, Result, Trigger, async_callback, impl_display_debug, new_interval_callback,
};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::interval;

async_callback!(WindowChangeCallback<T>);

/// WindowTrigger trigger when the current window changes.
pub struct WindowTrigger {
    last_window: Option<WindowIdentifier>,
    callback: WindowChangeCallback<WindowIdentifier>,
}

impl WindowTrigger {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(WindowIdentifier) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            last_window: None,
            callback: new_interval_callback(f),
        }
    }
}

#[async_trait]
impl Trigger for WindowTrigger {
    async fn start(&mut self) -> Result<()> {
        let mut ticker = interval(Duration::from_millis(500));

        loop {
            ticker.tick().await;
            if let Some(window) = get_current_window_identifier() {
                if self.last_window.as_ref() != Some(&window) {
                    self.last_window = Some(window.clone());
                    (self.callback)(window).await?;
                }
            }
        }
    }

    fn name(&self) -> String {
        "WindowTrigger".to_string()
    }
}
