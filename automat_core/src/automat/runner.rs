use crate::{await_shutdown, Automat, ErrorHandler, TriggerEvent, TriggerRuntime};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;

impl Automat {
  pub async fn run(self) -> crate::Result<()> {
    let error_handler = self.error_handler.clone();
    let mut trigger_handles = Vec::new();
    let mut event_handles = Vec::new();

    let shutdown_token = CancellationToken::new();

    for mut trigger in self.triggers {
      let (tx, rx) = channel(100);
      let handler = error_handler.clone();

      let shutdown_for_trigger = shutdown_token.clone();
      let shutdown_for_events = shutdown_token.clone();

      let rt = TriggerRuntime {
        tx: tx.clone(),
        shutdown: shutdown_for_trigger.clone(),
      };

      let trigger_handle = tokio::spawn(async move {
        let res = trigger.start(rt).await;
        if let Err(err) = res {
          let _ = tx.send(TriggerEvent::ErrorFatal(err)).await;
        } else {
          // If a trigger exits cleanly without a shutdown, report it as a stop.
          if !shutdown_for_trigger.is_cancelled() {
            let _ = tx.send(TriggerEvent::Stop).await;
          }
        }
      });

      let event_handle =
        tokio::spawn(
          async move { Self::handle_trigger_events(rx, handler, shutdown_for_events).await },
        );

      trigger_handles.push(trigger_handle);
      event_handles.push(event_handle);
    }

    let shutdown_result = tokio::select! {
      r = await_shutdown() => r,
      _ = shutdown_token.cancelled() => Ok(()),
    };

    shutdown_token.cancel();

    for trigger_handle in trigger_handles {
      let mut trigger_handle = trigger_handle;
      match timeout(Duration::from_secs(2), &mut trigger_handle).await {
        Ok(join_result) => {
          let _ = join_result;
        }
        Err(_) => {
          trigger_handle.abort();
          let _ = trigger_handle.await;
        }
      }
    }

    for event_handle in event_handles {
      let _ = event_handle.await;
    }

    shutdown_result
  }

  async fn handle_trigger_events(
    mut rx: Receiver<TriggerEvent>,
    error_handler: Option<ErrorHandler>,
    shutdown_token: CancellationToken,
  ) {
    while let Some(event) = rx.recv().await {
      match event {
        TriggerEvent::Error(err) => {
          if let Some(ref handler) = error_handler {
            handler(err);
          } else {
            eprintln!("Trigger error: {}", err);
          }
        }
        TriggerEvent::ErrorFatal(err) => {
          if let Some(ref handler) = error_handler {
            handler(err);
          } else {
            eprintln!("Fatal trigger error: {}", err);
          }
          shutdown_token.cancel();
        }
        TriggerEvent::Stop => {
          shutdown_token.cancel();
        }
      }
    }
  }
}
