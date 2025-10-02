use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    KeyboardAction::text("Hello!!!").run()?;

    await_shutdown().await
}
