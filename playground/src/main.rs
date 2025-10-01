use automat_core::*;

pub struct KeybindTrigger {
    key: char,
}

#[async_trait]
impl Trigger for KeybindTrigger {
    async fn start(&mut self) -> Result<()> {
        loop {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {
                    if input.trim().starts_with(self.key) {
                        self.run().await?;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        println!("Key pressed!");
        Ok(())
    }

    fn name(&self) -> &str {
        "keybind"
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    new_trigger(KeybindTrigger { key: 'a' }).await;

    await_shutdown().await
}
