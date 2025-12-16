use crate::{FileSystemTrigger, Result, TriggerContext};
use notify::{Config, Event};
use std::future::Future;
use std::path::Path;

/// Builder for configuring a `FileSystemTrigger` before adding it to `Automat`.
pub struct FileSystemBuilder {
  paths: Vec<(std::path::PathBuf, bool)>,
  config: Option<Config>,
}

impl FileSystemBuilder {
  /// Creates a new `FileSystemBuilder`.
  pub fn new() -> Self {
    Self {
      paths: Vec::new(),
      config: None,
    }
  }

  /// Adds a path to watch for file system changes.
  pub fn watch<P: AsRef<Path>>(mut self, path: P, recursive: bool) -> Self {
    self.paths.push((path.as_ref().to_path_buf(), recursive));
    self
  }

  /// Adds multiple paths to watch.
  pub fn watch_many<I, P>(mut self, paths: I) -> Self
  where
    I: IntoIterator<Item = (P, bool)>,
    P: AsRef<Path>,
  {
    for (path, recursive) in paths {
      self.paths.push((path.as_ref().to_path_buf(), recursive));
    }
    self
  }

  /// Watches a directory recursively (including all subdirectories).
  pub fn watch_recursive<P: AsRef<Path>>(self, path: P) -> Self {
    self.watch(path, true)
  }

  /// Watches a path non-recursively (does not monitor subdirectories).
  pub fn watch_non_recursive<P: AsRef<Path>>(self, path: P) -> Self {
    self.watch(path, false)
  }

  /// Sets a custom configuration for the file system watcher.
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }

  /// Sets the callback and builds the `FileSystemTrigger`.
  pub fn on_event<F, Fut>(self, callback: F) -> FileSystemTrigger
  where
    F: Fn(TriggerContext<Result<Event>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
  {
    let mut trigger = FileSystemTrigger::new(callback);

    if let Some(config) = self.config {
      trigger = trigger.with_config(config);
    }

    for (path, recursive) in self.paths {
      trigger = trigger.watch_path(path, recursive);
    }

    trigger
  }

  /// Sets a synchronous (blocking) callback and builds the `FileSystemTrigger`.
  pub fn on_event_blocking<F>(self, callback: F) -> FileSystemTrigger
  where
    F: Fn(TriggerContext<Result<Event>>) -> Result<()> + Send + Sync + 'static,
  {
    let mut trigger = FileSystemTrigger::new_blocking(callback);

    if let Some(config) = self.config {
      trigger = trigger.with_config(config);
    }

    for (path, recursive) in self.paths {
      trigger = trigger.watch_path(path, recursive);
    }

    trigger
  }

  /// Returns the number of paths currently configured to watch.
  pub fn watch_count(&self) -> usize {
    self.paths.len()
  }
}

impl Default for FileSystemBuilder {
  fn default() -> Self {
    Self::new()
  }
}
