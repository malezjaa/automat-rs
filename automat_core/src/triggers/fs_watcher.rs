use crate::{callback, impl_display_debug, Error, Result, Trigger, TriggerEvent};
use async_trait::async_trait;
use notify::{Config, Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

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
  /// Creates a new `FileSystemTrigger` with the given callback.
  ///
  /// # Arguments
  ///
  /// * `f` - A sync callback function that receives file system events and returns a `Result`.
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(Result<Event>) -> Result<()> + Send + Sync + 'static,
  {
    Self {
      callback: new_file_system_callback(f),
      config: None,
      watch_paths: Vec::new(),
    }
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
  /// Each file system event triggers the registered callback.
  async fn start(&mut self, tx: Sender<TriggerEvent>) {
    use notify::RecommendedWatcher;
    use std::sync::mpsc;

    if self.watch_paths.is_empty() {
      let _ = tx
        .send(TriggerEvent::ErrorFatal(Error::NoWatchPaths()))
        .await;

      return;
    }

    let (fs_tx, fs_rx) = mpsc::channel();
    let mut watcher = match RecommendedWatcher::new(fs_tx, self.config.unwrap_or_default()) {
      Ok(w) => w,
      Err(e) => {
        let _ = tx.send(TriggerEvent::ErrorFatal(Error::from(e))).await;
        return;
      }
    };

    for (path, mode) in &self.watch_paths {
      if let Err(e) = watcher.watch(path, *mode) {
        let _ = tx.send(TriggerEvent::Error(Error::from(e))).await;
      }
    }

    for res in fs_rx {
      if let Err(err) = (self.callback)(res.map_err(Into::into)) {
        let _ = tx.send(TriggerEvent::Error(err)).await;
      }
    }
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
