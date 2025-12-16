use automat_core::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
  Automat::new()
    .on_window_focus(async |ctx| {
      let window = ctx.data;
      let title = window.executable_path().unwrap_or_default();
      if title.contains("Notepad.exe") {
        sleep(Duration::from_secs(2)).await;
        MinimizeWindow::from_id(window.id()).run()?;
        println!("Closed Notepad window with ID: {:?}", window.id());
      }

      Ok(())
    })
    .run()
    .await
}
