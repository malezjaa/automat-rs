use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
  Automat::new()
    .on_error(|err| {
      eprintln!("🚨 Custom error handler: {}", err);
    })
    .on_process(process)
    .run()
    .await
}

fn process(ctx: TriggerContext<ProcessEvent>) -> Result<()> {
  match &ctx.data {
    ProcessEvent::Started(st) => {}
    _ => {}
  }

  Ok(())
}
