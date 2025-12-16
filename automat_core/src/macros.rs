/// Generates an async-first API and a blocking variant.
///
/// Two modes:
/// - `method`: emits builder-style methods that push a trigger and return `Self`.
/// - `assoc`: emits associated fns returning `Self`.
///
/// The simplified forms take a single definition and generate the `_blocking` variant
/// automatically. You specify the callback signature via `callback(...)`.
///
/// Example (method):
///
/// ```ignore
/// pair_api! {
///   method
///     /// Async-first. Blocking variant is auto-generated as `on_interval_blocking`.
///     on_interval(interval: Duration, f: F)
///       callback(Duration)
///       => (IntervalTrigger)::new(interval, f);
/// }
/// ```
///
/// Example (assoc):
///
/// ```ignore
/// pair_api! {
///   assoc
///     /// Async-first.
///     new(arg: T, f: F)
///       callback(T)
///       async => Self { /* ... */ };
///       /// Blocking.
///       blocking => Self { /* ... */ };
/// }
/// ```
#[macro_export]
macro_rules! pair_api {
  // --- Shorthand forms (single definition, auto-generates `_blocking`) ---
  //
  // These forms aim to remove the repetitive `<F, Fut>` + `where { ... }` boilerplate.
  // You provide:
  // - the function signature (must include `f: F`)
  // - the callback input types via `callback(...)`
  // - either:
  //   - a base ctor path `Ty::ctor(...)` (blocking uses `Ty::ctor_blocking(...)`), OR
  //   - explicit `async => ...; blocking => ...;` bodies.

  (
    method
      $(#[$doc_async:meta])*
      $name:ident($($args:tt)*)
        callback ( $($cb_args:ty),* $(,)? )
        => ( $ty:path ) :: $ctor:ident ( $($ctor_args:tt)* );
  ) => {
    ::paste::paste! {
      $(#[$doc_async])*
      pub fn $name<F, Fut>(mut self, $($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> Fut + Send + Sync + 'static,
        Fut: ::std::future::Future<Output = $crate::Result<()>> + Send + 'static,
      {
        let trigger = $ty::$ctor($($ctor_args)*);
        self.triggers.push(Box::new(trigger));
        self
      }

      #[doc = concat!("Blocking variant of `", stringify!($name), "`.")]
      pub fn [<$name _blocking>]<F>(mut self, $($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> $crate::Result<()> + Send + Sync + 'static,
      {
        let trigger = $ty::[<$ctor _blocking>]($($ctor_args)*);
        self.triggers.push(Box::new(trigger));
        self
      }
    }
  };

  (
    assoc
      $(#[$doc_async:meta])*
      $name:ident($($args:tt)*)
        callback ( $($cb_args:ty),* $(,)? )
        => ( $ty:path ) :: $ctor:ident ( $($ctor_args:tt)* );
  ) => {
    ::paste::paste! {
      $(#[$doc_async])*
      pub fn $name<F, Fut>($($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> Fut + Send + Sync + 'static,
        Fut: ::std::future::Future<Output = $crate::Result<()>> + Send + 'static,
      {
        $ty::$ctor($($ctor_args)*)
      }

      #[doc = concat!("Blocking variant of `", stringify!($name), "`.")]
      pub fn [<$name _blocking>]<F>($($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> $crate::Result<()> + Send + Sync + 'static,
      {
        $ty::[<$ctor _blocking>]($($ctor_args)*)
      }
    }
  };

  (
    method
      $(#[$doc_async:meta])*
      $name:ident($($args:tt)*)
        callback ( $($cb_args:ty),* $(,)? )
        async => $body_async:expr;
        $(#[$doc_block:meta])*
        blocking => $body_block:expr;
  ) => {
    ::paste::paste! {
      $(#[$doc_async])*
      pub fn $name<F, Fut>(mut self, $($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> Fut + Send + Sync + 'static,
        Fut: ::std::future::Future<Output = $crate::Result<()>> + Send + 'static,
      {
        let trigger = $body_async;
        self.triggers.push(Box::new(trigger));
        self
      }

      $(#[$doc_block])*
      #[doc = concat!("Blocking variant of `", stringify!($name), "`.")]
      pub fn [<$name _blocking>]<F>(mut self, $($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> $crate::Result<()> + Send + Sync + 'static,
      {
        let trigger = $body_block;
        self.triggers.push(Box::new(trigger));
        self
      }
    }
  };

  (
    assoc
      $(#[$doc_async:meta])*
      $name:ident($($args:tt)*)
        callback ( $($cb_args:ty),* $(,)? )
        async => $body_async:expr;
        $(#[$doc_block:meta])*
        blocking => $body_block:expr;
  ) => {
    ::paste::paste! {
      $(#[$doc_async])*
      pub fn $name<F, Fut>($($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> Fut + Send + Sync + 'static,
        Fut: ::std::future::Future<Output = $crate::Result<()>> + Send + 'static,
      {
        $body_async
      }

      $(#[$doc_block])*
      #[doc = concat!("Blocking variant of `", stringify!($name), "`.")]
      pub fn [<$name _blocking>]<F>($($args)*) -> Self
      where
        F: Fn($($cb_args),*) -> $crate::Result<()> + Send + Sync + 'static,
      {
        $body_block
      }
    }
  };
}
