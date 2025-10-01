use std::future::Future;
use std::pin::Pin;

/// Macro to generate async callback types with custom names and arguments
#[macro_export]
macro_rules! async_callback {
    ($name:ident $(<$($t:ident),+>)?) => {
        #[allow(dead_code)]
        pub type $name$(<$($t: Send + 'static),+>)? =
            Box<dyn Fn($($($t),+)?) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

        paste::paste! {
            #[allow(dead_code)]
            pub fn [<new_ $name:snake>]<F, Fut $($(, $t)+)?>(f: F) -> $name$(<$($t),+>)?
            where
                F: Fn($($($t),+)?) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Result<()>> + Send + 'static,
                $($($t: Send + 'static),+)?
            {
                Box::new(move |$($([<arg_ $t:lower>]),+)?| Box::pin(f($($([<arg_ $t:lower>]),+)?)))
            }
        }
    };
}
