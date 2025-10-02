use automat_core::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    await_shutdown().await
}
