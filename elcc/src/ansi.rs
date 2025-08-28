//! ANSI styling.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether to use ANSI styling on stdout.
static ANSI_STDOUT: AtomicBool = AtomicBool::new(false);

/// Whether to use ANSI styling on stderr.
static ANSI_STDERR: AtomicBool = AtomicBool::new(false);

/// Set ANSI styling on stdout.
pub fn set_ansi_stdout(b: bool) {
    ANSI_STDOUT.store(b, Ordering::Release);
}

/// Set ANSI styling on stderr.
pub fn set_ansi_stderr(b: bool) {
    ANSI_STDERR.store(b, Ordering::Release);
}

/// Get ANSI styling on stdout.
pub fn get_ansi_stdout() -> bool {
    ANSI_STDOUT.load(Ordering::Acquire)
}

/// Get ANSI styling on stderr.
pub fn get_ansi_stderr() -> bool {
    ANSI_STDERR.load(Ordering::Acquire)
}

/// Macro applying [`cstr`] or [`untagged`] to the first argument depending on [`get_ansi_stdout`].
#[macro_export]
macro_rules! xapply_stdout {
    ($macro:path; $str:tt $($args:tt)*) => {
        if crate::ansi::get_ansi_stdout() {
            $macro!(::color_print::cstr!($str) $($args)*)
        } else {
            $macro!(::color_print::untagged!($str) $($args)*)
        }
    };

}

/// Macro application with conditional ANSI styling on stdout.
#[macro_export]
macro_rules! xapply_stderr {
    ($macro:path; $str:tt $($args:tt)*) => {
        if crate::ansi::get_ansi_stderr() {
            $macro!(::color_print::cstr!($str) $($args)*)
        } else {
            $macro!(::color_print::untagged!($str) $($args)*)
        }
    };
}

/// [`println`] with conditional ANSI styling.
#[macro_export]
macro_rules! xprintln {
    ($($args:tt)+) => {
        crate::xapply_stdout!(::std::println; $($args)+)
    };
}

/// [`eprintln`] with conditional ANSI styling.
#[macro_export]
macro_rules! xeprintln {
    ($($args:tt)+) => {
        crate::xapply_stderr!(::std::eprintln; $($args)+)
    };
}

/// [`print`] with conditional ANSI styling.
#[macro_export]
macro_rules! xprint {
    ($($args:tt)+) => {
        crate::xapply_stdout!(::std::print; $($args)+)
    };
}

/// [`eprint`] with conditional ANSI styling.
#[macro_export]
macro_rules! xeprint {
    ($($args:tt)+) => {
        crate::xapply_stderr!(::std::eprint; $($args)+)
    };
}
