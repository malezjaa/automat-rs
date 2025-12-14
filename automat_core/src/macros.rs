/// Generates an async-first API and a blocking variant.
///
/// Two modes:
///
/// - `method`: emits builder-style methods that push a trigger and return `Self`.
/// - `assoc`: emits associated fns returning `Self`.
///
/// # Example (method)
///
/// ```ignore
/// pair_api! {
///   method
///     /// Async-first.
///     on_interval<F, Fut>(interval: Duration, f: F)
///       where { F: Fn(Duration) -> Fut + Send + Sync + 'static, Fut: Future<Output = Result<()>> + Send + 'static }
///       => IntervalTrigger::new(interval, f);
///     /// Blocking.
///     on_interval_blocking<F>(interval: Duration, f: F)
///       where { F: Fn(Duration) -> Result<()> + Send + Sync + 'static }
///       => IntervalTrigger::new_blocking(interval, f);
/// }
/// ```
///
/// # Example (assoc)
///
/// ```ignore
/// pair_api! {
///   assoc
///     new<F, Fut>(arg: T, f: F)
///       where { F: Fn(T) -> Fut + Send + Sync + 'static, Fut: Future<Output = Result<()>> + Send + 'static }
///       => Self { /* ... */ };
///     new_blocking<F>(arg: T, f: F)
///       where { F: Fn(T) -> Result<()> + Send + Sync + 'static }
///       => Self { /* ... */ };
/// }
/// ```
#[macro_export]
macro_rules! pair_api {
  (
    method
      $(#[$doc_async:meta])*
      $async_name:ident<$($gen_async:ident),+>($($args_async:tt)*)
        where { $($where_async:tt)* }
        => $ctor_async:expr;
      $(#[$doc_block:meta])*
      $block_name:ident<$($gen_block:ident),+>($($args_block:tt)*)
        where { $($where_block:tt)* }
        => $ctor_block:expr;
  ) => {
    $(#[$doc_async])*
    pub fn $async_name<$($gen_async),+>(mut self, $($args_async)*) -> Self
    where
      $($where_async)*
    {
      let trigger = $ctor_async;
      self.triggers.push(Box::new(trigger));
      self
    }

    $(#[$doc_block])*
    pub fn $block_name<$($gen_block),+>(mut self, $($args_block)*) -> Self
    where
      $($where_block)*
    {
      let trigger = $ctor_block;
      self.triggers.push(Box::new(trigger));
      self
    }
  };

  (
    assoc
      $(#[$doc_async:meta])*
      $async_name:ident<$($gen_async:ident),+>($($args_async:tt)*)
        where { $($where_async:tt)* }
        => $body_async:expr;
      $(#[$doc_block:meta])*
      $block_name:ident<$($gen_block:ident),+>($($args_block:tt)*)
        where { $($where_block:tt)* }
        => $body_block:expr;
  ) => {
    $(#[$doc_async])*
    pub fn $async_name<$($gen_async),+>($($args_async)*) -> Self
    where
      $($where_async)*
    {
      $body_async
    }

    $(#[$doc_block])*
    pub fn $block_name<$($gen_block),+>($($args_block)*) -> Self
    where
      $($where_block)*
    {
      $body_block
    }
  };
}
