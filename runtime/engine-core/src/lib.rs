pub mod log;

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::log::debug(&format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::info(&format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::log::warn(&format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::error(&format!($($arg)*)) };
}
