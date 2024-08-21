#[macro_export]
macro_rules! asrt {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            assert!($($arg)*);
        }
    };
}
