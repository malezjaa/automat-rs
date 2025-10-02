use automat_core::*;
use notify_rust::{Hint, Notification, Timeout};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    new_trigger(vec![IntervalTrigger::new(
        Duration::from_secs(4),
        |_| async move {
            println!("current titlebar {:?}", get_current_window_title());
            Ok(())
        },
    )])
    .await;

    await_shutdown().await
}
