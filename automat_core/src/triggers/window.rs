use crate::window::get_current_window_identifier;
use crate::{callback, Result, Trigger, Window};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::interval;

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
    async fn start(&mut self) -> Result<()> {
        let mut ticker = interval(Duration::from_millis(500));

        loop {
            ticker.tick().await;
            if let Some(window) = get_current_window_identifier() {
                let new_window = Window::new(window);
                if self.last_window.as_ref() != Some(&new_window) {
                    self.last_window = Some(new_window.clone());
                    (self.callback)(new_window)?;
                }
            }
        }
    }

    fn name(&self) -> String {
        "WindowTrigger".to_string()
    }
}
