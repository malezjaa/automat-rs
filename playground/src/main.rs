use automat_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    Automat::new()
        .on_process(|event| {
            match event {
                ProcessEvent::Started(st) => {
                    println!("Process started {}", st.name);
                }
                ProcessEvent::Exited(ex) => {
                    println!("Process exited: {}", ex.name);
                }
            }
            Ok(())
        })
        .run()
        .await
}
