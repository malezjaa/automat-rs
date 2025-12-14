use automat_core::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
  Automat::new()
    .on_error(|err| {
      eprintln!("🚨 Custom error handler: {}", err);
    })
    .extend(fs_watcher())
    .run()
    .await
}

fn fs_watcher() -> Automat {
  Automat::new().with_fs_watch(|builder| {
    builder
      .watch_recursive(Path::new("automat_core").canonicalize().unwrap())
      .on_event(async |event| {
        println!("🔧 File event: {:?}", event);
        Ok(())
      })
  })
}
