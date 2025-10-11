use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    let displays = DisplayInfo::all()?;

    await_shutdown().await
}
