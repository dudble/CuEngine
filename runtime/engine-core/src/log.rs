use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

static MAX_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);

pub fn set_level(level: Level) {
    MAX_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn debug(message: &str) {
    emit(Level::Debug, message);
}

pub fn info(message: &str) {
    emit(Level::Info, message);
}

pub fn warn(message: &str) {
    emit(Level::Warn, message);
}

pub fn error(message: &str) {
    emit(Level::Error, message);
}

fn emit(level: Level, message: &str) {
    if (level as u8) < MAX_LEVEL.load(Ordering::Relaxed) {
        return;
    }
    let (hour, minute, second) = utc_hms();
    println!(
        "[{:02}:{:02}:{:02}] {:5} {}",
        hour,
        minute,
        second,
        tag(level),
        message
    );
}

fn tag(level: Level) -> &'static str {
    match level {
        Level::Debug => "DEBUG",
        Level::Info => "INFO",
        Level::Warn => "WARN",
        Level::Error => "ERROR",
    }
}

fn utc_hms() -> (u64, u64, u64) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    ((seconds / 3600) % 24, (seconds / 60) % 60, seconds % 60)
}
