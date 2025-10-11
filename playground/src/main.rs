use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;

    new_trigger!(
        FileSystemTrigger::new(|event| async move {
            println!("Change: {event:?}");
            Ok(())
        })
        .watch_path(cwd, true)
    );
    await_shutdown().await
}
