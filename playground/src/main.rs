use automat_core::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let interval = IntervalTrigger::new(Duration::from_secs(5), |duration| async move {
        println!("Interval: {:?}", duration);
        Ok(())
    });

    new_trigger([interval]).await;
    await_shutdown().await
}
