use automat_core::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
  Automat::new()
    .on_clipboard_change(async |ctx| {
        println!("Clipboard changed: {}", ctx.data.content());
        Ok(())
    })
    .run()
    .await
}
