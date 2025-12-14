use crate::{callback, impl_display_debug, pair_api, send_error, Error, Result, Trigger, TriggerEvent, TriggerRuntime};
use async_trait::async_trait;
use notify::{Config, Event, EventHandler, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::future::Future;
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
///
/// `FileSystemTrigger` monitors one or more file system paths for changes such as file
/// modifications, creations, deletions, and permission changes. When an event is detected,
/// the registered callback is invoked with the event details.
pub struct FileSystemTrigger {
  callback: FileSystemCallback<Result<Event>>,
  config: Option<Config>,
  watch_paths: Vec<(PathBuf, RecursiveMode)>,
}

impl FileSystemTrigger {
  pair_api! {
    assoc
      /// Creates a new `FileSystemTrigger` with the given callback.
      ///
      /// # Arguments
      ///
      /// * `f` - An async callback function that receives file system events and returns a `Result`.
      new<F, Fut>(f: F)
        where {
          F: Fn(Result<Event>) -> Fut + Send + Sync + 'static,
          Fut: Future<Output = Result<()>> + Send + 'static,
        }
        => Self { callback: new_file_system_callback(f), config: None, watch_paths: Vec::new() };
      /// Creates a new `FileSystemTrigger` with a synchronous (blocking) callback.
      new_blocking<F>(f: F)
        where {
          F: Fn(Result<Event>) -> Result<()> + Send + Sync + 'static,
        }
        => Self { callback: new_file_system_callback_blocking(f), config: None, watch_paths: Vec::new() };
  }

  /// Configures the watcher with custom settings.
  ///
  /// # Arguments
  ///
  /// * `config` - A `notify::Config` an object with custom watcher settings.
  ///   If not provided, the default configuration will be used.
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  /// Adds a path to be monitored for file system events.
  ///
  /// # Arguments
  ///
  /// * `path` - The file system path to watch.
  /// * `recursive` - If `true`, watches the directory and all its subdirectories.
  ///   If `false`, only watches the immediate directory.
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

    println!("started: {:?}", self.watch_paths);

    let (fs_tx, mut fs_rx) = tokio::sync::mpsc::channel::<notify::Result<Event>>(1024);
    let handler = TokioEventHandler { tx: fs_tx };

    let mut watcher = match RecommendedWatcher::new(handler, self.config.clone().unwrap_or_default()) {
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

          if let Err(err) = (self.callback)(res.map_err(Into::into)).await {
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
