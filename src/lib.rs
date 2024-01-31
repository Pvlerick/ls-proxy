#[macro_export]
macro_rules! info {
    ($arg:tt) => {
        tracing::event!(Level::INFO, $arg);
    };
    ($($arg:tt)*) => {
        tracing::event!(Level::INFO, format!($($arg)*));
    };
}
