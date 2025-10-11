/// Macro to generate sync callback types with custom names and arguments
#[macro_export]
macro_rules! callback {
    ($name:ident $(<$($t:ident),+>)?) => {
        #[allow(dead_code)]
        pub type $name$(<$($t: Send + 'static),+>)? =
            Box<dyn Fn($($($t),+)?) -> Result<()> + Send + Sync>;

        paste::paste! {
            #[allow(dead_code)]
            pub fn [<new_ $name:snake>]<F $($(, $t)+)?>(f: F) -> $name$(<$($t),+>)?
            where
                F: Fn($($($t),+)?) -> Result<()> + Send + Sync + 'static,
                $($($t: Send + 'static),+)?
            {
                Box::new(f)
            }
        }
    };
}
