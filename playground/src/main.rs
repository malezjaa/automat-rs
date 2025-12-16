use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
  Automat::new()
    .on_window_focus(async |ctx| {
      let window = ctx.data;
      let title = format!(
        "Focused: {}",
        window.title().unwrap_or("Untitled".to_string())
      );
      let action = SetWindowTitle::for_window(window, &title);
      action.run()?;

      println!("{:?}", window.title());

      Ok(())
    })
    .run()
    .await
}
