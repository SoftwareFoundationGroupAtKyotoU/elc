//! Logging.

use crate::ansi::{DIM, RESET};
use std::fmt::Arguments;
use std::sync::OnceLock;

/// Level of a log.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum LogLevel {
    /// Error.
    Error = 0,
    /// Warning.
    Warn = 1,
    /// Information.
    Info = 2,
    /// Report.
    Report = 3,
    /// Debug.
    Debug = 4,
}

/// Filter of a log.
#[derive(Debug, Clone, Copy)]
pub enum LogFilter {
    /// No log at all.
    Off = 0,
    /// Error.
    Error = 1,
    /// Warning.
    Warn = 2,
    /// Information.
    Info = 3,
    /// Report.
    Report = 4,
    /// Debug.
    Debug = 5,
}

impl LogFilter {
    /// Judges if logs of the level are enabled under the filter.
    #[inline]
    fn enables(self, log_level: LogLevel) -> bool {
        (log_level as u8) < (self as u8)
    }
}

/// Turns [`i32`] into [`LogFilter`].
impl From<i32> for LogFilter {
    fn from(value: i32) -> Self {
        match value {
            1 => LogFilter::Error,
            2 => LogFilter::Warn,
            3 => LogFilter::Info,
            4 => LogFilter::Report,
            _ if value <= 0 => LogFilter::Off,
            _ => LogFilter::Debug,
        }
    }
}

/// Global log filter.
static LOG_FILTER: OnceLock<LogFilter> = OnceLock::new();

/// Initialize sthe global log filter.
pub fn init_log_filter(log_filter: LogFilter) {
    LOG_FILTER.set(log_filter).unwrap_or_else(|old_log_filter| {
        panic!(
            "The global log filter has already been initialized to {:?}",
            old_log_filter
        )
    })
}

/// Gets the global log filter.
#[inline]
pub fn get_log_filter() -> LogFilter {
    *LOG_FILTER
        .get()
        .expect("The global log filter has not been set")
}

impl LogLevel {
    /// Checks if the log level is enabled.
    #[inline]
    pub fn is_enabled(self) -> bool {
        get_log_filter().enables(self)
    }
}

/// Outputs a log. Function version of the [`log!`] macro.
pub fn log(log_level: LogLevel, args: Arguments) {
    if log_level.is_enabled() {
        match log_level {
            LogLevel::Debug => {
                eprintln!("{DIM}# {}{RESET}", args);
            }
            _ => eprintln!("{}", args),
        }
    }
}

/// Outputs a log of the specified level.
#[macro_export]
macro_rules! log {
    ($log_level:expr, $($args:tt)*) => {
        crate::log::log($log_level, format_args!($($args)*))
    };
}

/// Outputs an error log.
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => { crate::log!(crate::log::LogLevel::Error, $($args)*) }
}

/// Outputs an warning log.
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => { crate::log!(crate::log::LogLevel::Warn, $($args)*) }
}

/// Output an information log.
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => { crate::log!(crate::log::LogLevel::Info, $($args)*) }
}

/// Outputs a report log.
#[macro_export]
macro_rules! report {
    ($($args:tt)*) => { crate::log!(crate::log::LogLevel::Report, $($args)*) }
}

/// Outputs a debug log.
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => { crate::log!(crate::log::LogLevel::Debug, $($args)*) }
}
