/// Implements both Display and Debug traits with the same formatting
#[macro_export]
macro_rules! impl_display_debug {
    ($type:ty, |$self:ident, $f:ident| $body:expr) => {
        impl std::fmt::Display for $type {
            fn fmt(&$self, $f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $body
            }
        }

        impl std::fmt::Debug for $type {
            fn fmt(&$self, $f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $body
            }
        }
    };
}
