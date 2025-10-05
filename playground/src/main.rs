use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    new_trigger!(WindowTrigger::new(|id| async move {
        println!("current window {:?}", get_window_state(id));
        Ok(())
    }));

    await_shutdown().await
}
