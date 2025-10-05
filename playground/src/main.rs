use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    await_shutdown().await
}
