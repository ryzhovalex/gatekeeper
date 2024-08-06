#[macro_export]
macro_rules! asrt {
    ($($arg:tt)*) => {
        if $crate::cfg!(debug_assertions) {
            $crate::assert!($($arg)*);
        }
    };
}
