use crate::{
  callback, impl_display_debug, pair_api, send_error, Error, Result, Trigger,
  TriggerContext, TriggerEvent, TriggerRuntime,
};
use async_trait::async_trait;
use notify::{Config, Event, EventHandler, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
struct TokioEventHandler {
  tx: Sender<notify::Result<Event>>,
}

impl EventHandler for TokioEventHandler {
  fn handle_event(&mut self, event: notify::Result<Event>) {
    let _ = self.tx.blocking_send(event);
  }
}

callback!(FileSystemCallback<T>);

/// A trigger that watches for file system events and executes a callback when events occur.
pub struct FileSystemTrigger {
  callback: FileSystemCallback<TriggerContext<Result<Event>>>,
  config: Option<Config>,
  watch_paths: Vec<(PathBuf, RecursiveMode)>,
}

impl FileSystemTrigger {
  pair_api! {
    assoc
      /// Creates a new `FileSystemTrigger` with the given callback.
      new(f: F)
        callback(TriggerContext<Result<Event>>)
        async => Self { callback: new_file_system_callback(f), config: None, watch_paths: Vec::new() };
        /// Creates a new `FileSystemTrigger` with a synchronous (blocking) callback.
        blocking => Self { callback: new_file_system_callback_blocking(f), config: None, watch_paths: Vec::new() };
  }

  /// Configures the watcher with custom settings.
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  /// Adds a path to be monitored for file system events.
  pub fn watch_path(mut self, path: PathBuf, recursive: bool) -> Self {
    self.watch_paths.push((
      path,
      if recursive {
        RecursiveMode::Recursive
      } else {
        RecursiveMode::NonRecursive
      },
    ));
    self
  }

  pub fn watch_count(&self) -> usize {
    self.watch_paths.len()
  }
}

#[async_trait]
impl Trigger for FileSystemTrigger {
  /// Starts watching the configured paths and executes the callback on each event.
  ///
  /// This method blocks until an error occurs or the watcher is stopped.
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
    use notify::RecommendedWatcher;

    if self.watch_paths.is_empty() {
      let _ = rt
        .tx
        .send(TriggerEvent::ErrorFatal(Error::NoWatchPaths()))
        .await;

      return Ok(());
    }
    let (fs_tx, mut fs_rx) = tokio::sync::mpsc::channel::<notify::Result<Event>>(1024);
    let handler = TokioEventHandler { tx: fs_tx };

    let mut watcher =
      match RecommendedWatcher::new(handler, self.config.clone().unwrap_or_default()) {
        Ok(w) => w,
        Err(e) => {
          send_error(&rt.tx, Error::from(e), "FileSystemTrigger").await;

          return Ok(());
        }
      };

    for (path, mode) in &self.watch_paths {
      if let Err(e) = watcher.watch(path, *mode) {
        if !send_error(&rt.tx, Error::from(e), "FileSystemTrigger").await {
          return Ok(());
        }
      }
    }

    loop {
      tokio::select! {
        _ = rt.shutdown.cancelled() => break,
        maybe = fs_rx.recv() => {
          let Some(res) = maybe else {
            return Err(Error::FileWatcherStopped);
          };

          let ctx=  TriggerContext::new(res.map_err(Into::into), rt.tx.clone());
          if let Err(err) = (self.callback)(ctx).await {
            if !send_error(&rt.tx, err, "FileSystemTrigger").await {
              break;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn name(&self) -> String {
    "FileSystemTrigger".to_string()
  }
}

impl_display_debug!(FileSystemTrigger, |self, f| write!(
  f,
  "{} (watching {} path{})",
  self.name(),
  self.watch_count(),
  if self.watch_count() == 1 { "" } else { "s" }
));
