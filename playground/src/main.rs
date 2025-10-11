use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;

    println!(
        "{:?}",
        FileSystemTrigger::new(|event| {
            println!("Change: {event:?}");
            Ok(())
        })
        .watch_path(cwd, true)
    );
    await_shutdown().await
}
