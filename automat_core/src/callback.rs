/// Macro to generate async-first callback types.
///
/// Supported forms:
/// - `callback!(MyCallback);` for a zero-arg callback
/// - `callback!(MyCallback<T>);` for a single-arg callback
///
/// The generated callback type is a boxed `Fn(..) -> Future<Output = Result<()>>`.
/// For convenience, a `_blocking` constructor is also generated which wraps a
/// synchronous `Fn(..) -> Result<()>` into an async callback.
#[macro_export]
macro_rules! callback {
    ($name:ident) => {
        #[allow(dead_code)]
        pub type $name = Box<
            dyn Fn() -> ::std::pin::Pin<
                Box<dyn ::std::future::Future<Output = Result<()>> + Send + 'static>
            > + Send + Sync
        >;

        paste::paste! {
            #[allow(dead_code)]
            pub fn [<new_ $name:snake>]<F, Fut>(f: F) -> $name
            where
                F: Fn() -> Fut + Send + Sync + 'static,
                Fut: ::std::future::Future<Output = Result<()>> + Send + 'static,
            {
                Box::new(move || ::std::boxed::Box::pin(f()))
            }

            #[allow(dead_code)]
            pub fn [<new_ $name:snake _blocking>]<F>(f: F) -> $name
            where
                F: Fn() -> Result<()> + Send + Sync + 'static,
            {
                Box::new(move || {
                    let result = f();
                    ::std::boxed::Box::pin(async move { result })
                })
            }
        }
    };

    ($name:ident<$t:ident>) => {
        #[allow(dead_code)]
        pub type $name<$t> = Box<
            dyn Fn($t) -> ::std::pin::Pin<
                Box<dyn ::std::future::Future<Output = Result<()>> + Send + 'static>
            > + Send + Sync
        >;

        paste::paste! {
            #[allow(dead_code)]
            pub fn [<new_ $name:snake>]<F, Fut, $t>(f: F) -> $name<$t>
            where
                F: Fn($t) -> Fut + Send + Sync + 'static,
                Fut: ::std::future::Future<Output = Result<()>> + Send + 'static,
                $t: Send + 'static,
            {
                Box::new(move |arg: $t| ::std::boxed::Box::pin(f(arg)))
            }

            #[allow(dead_code)]
            pub fn [<new_ $name:snake _blocking>]<F, $t>(f: F) -> $name<$t>
            where
                F: Fn($t) -> Result<()> + Send + Sync + 'static,
                $t: Send + 'static,
            {
                Box::new(move |arg: $t| {
                    let result = f(arg);
                    ::std::boxed::Box::pin(async move { result })
                })
            }
        }
    };
}
