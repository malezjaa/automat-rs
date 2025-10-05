use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    let windows = list_windows()?;
    println!("{:#?}", windows);

    new_trigger!(WindowTrigger::new(|id| async move {
        println!("current window {:?}", get_window_state(id));
        Ok(())
    }));

    await_shutdown().await
}
