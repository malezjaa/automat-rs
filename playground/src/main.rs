use automat_core::*;
use std::process::Command;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    await_shutdown().await
}
