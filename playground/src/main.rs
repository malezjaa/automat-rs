use automat_core::*;
use notify_rust::{Hint, Notification, Timeout};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    new_trigger!(WindowTrigger::new(|id| async move {
        println!("current window {:?}", get_window_title(id));
        Ok(())
    },));

    await_shutdown().await
}
