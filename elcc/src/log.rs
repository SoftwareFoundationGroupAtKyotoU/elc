//! Logging.

use crate::ansi::{BOLD, DIM, RED, RESET};
use std::fmt::Arguments;
use std::mem::transmute;
use std::sync::atomic::{AtomicI8, Ordering};

/// Level of a log.
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// Error.
    Error = -2,
    /// Warning.
    Warn = -1,
    /// Information.
    Info = 0,
    /// Report.
    Report = 1,
    /// Debug.
    Debug = 2,
}

/// Filter of a log.
#[derive(Debug, Clone, Copy, Default)]
pub enum LogFilter {
    /// Up to error logs.
    Error = -2,
    /// Up to warning logs.
    Warn = -1,
    /// Up to information logs.
    #[default]
    Info = 0,
    /// Up to report logs.
    Report = 1,
    /// Up to debug logs.
    Debug = 2,
}

impl LogFilter {
    /// Judges if logs of the level are enabled under the filter.
    #[inline]
    fn enables(self, log_level: LogLevel) -> bool {
        (log_level as i8) <= (self as i8)
    }
}

/// Error for turning [`LogFilter`] from an integer.
#[derive(Debug, Clone, Copy)]
pub enum LogFilterError {
    /// Too verbose.
    TooVerbose,
    /// Too quiet.
    TooQuiet,
}

/// Turns [`i8`] into [`LogFilter`].
impl TryFrom<i8> for LogFilter {
    type Error = LogFilterError;
    fn try_from(value: i8) -> Result<Self, LogFilterError> {
        match value {
            -2 => Ok(LogFilter::Error),
            -1 => Ok(LogFilter::Warn),
            0 => Ok(LogFilter::Info),
            1 => Ok(LogFilter::Report),
            2 => Ok(LogFilter::Debug),
            _ if value > 2 => Err(LogFilterError::TooVerbose),
            _ if value < -2 => Err(LogFilterError::TooQuiet),
            _ => unreachable!(),
        }
    }
}

/// Global log filter. Always set to a value of [`LogFilter`].
static LOG_FILTER: AtomicI8 = AtomicI8::new(LogFilter::Info as i8);
// Cannot use `LogFilter::default()` here because the function is not marked `const`.

/// Sets the global log filter.
#[inline]
pub fn set_log_filter(log_filter: LogFilter) {
    LOG_FILTER.store(log_filter as i8, Ordering::Release);
}

/// Gets the global log filter.
#[inline]
pub fn get_log_filter() -> LogFilter {
    unsafe { transmute(LOG_FILTER.load(Ordering::Acquire)) }
}

impl LogLevel {
    /// Checks if the log level is enabled.
    #[inline]
    pub fn is_enabled(self) -> bool {
        get_log_filter().enables(self)
    }
}

/// Utility macro for checking if the log level is enabled.
#[macro_export]
macro_rules! is_enabled {
    ($level:ident) => {
        crate::log::LogLevel::$level.is_enabled()
    };
}

/// Outputs an error log. Body of the [`error!`] macro.
#[inline]
pub fn output_error(args: Arguments) {
    eprintln!("{BOLD}{RED}Error{RESET}{BOLD}: {args}{RESET}");
}

/// Outputs an error log if enabled.
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        if crate::is_enabled!(Error) {
            crate::log::output_error(format_args!($($args)*));
        }
    }
}

/// Outputs a warning log. Body of the [`warn!`] macro.
#[inline]
pub fn output_warn(args: Arguments) {
    eprintln!("{BOLD}Warning: {args}{RESET}");
}

/// Outputs a warning log if enabled.
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {
        if crate::is_enabled!(Warn) {
            crate::log::output_warn(format_args!($($args)*));
        }
    }
}

/// Outputs an information log. Body of the [`info!`] macro.
#[inline]
pub fn output_info(args: Arguments) {
    eprintln!("{BOLD}Info{RESET}: {args}");
}

/// Outputs an information log if enabled.
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        if crate::is_enabled!(Info) {
            crate::log::output_info(format_args!($($args)*));
        }
    }
}

/// Outputs a report log. Body of the [`report!`] macro.
#[inline]
pub fn output_report(args: Arguments) {
    eprintln!("+ {args}");
}

/// Outputs a report log if enabled.
#[macro_export]
macro_rules! report {
    ($($args:tt)*) => {
        if crate::is_enabled!(Report) {
            crate::log::output_report(format_args!($($args)*));
        }
    }
}

/// Outputs a debug log. Body of the [`debug!`] macro.
#[inline]
pub fn output_debug(args: Arguments) {
    eprintln!("{DIM}# {args}{RESET}");
}

/// Outputs a debug log if enabled.
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        if crate::is_enabled!(Debug) {
            crate::log::output_debug(format_args!($($args)*));
        }
    }
}
