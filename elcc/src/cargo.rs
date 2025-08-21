//! For using cargo.

use crate::log::LogLevel;
use crate::report;
use crate::util::{AppliedTo as _, CommandExtra as _};
use std::process::Command;

/// Check if cargo should be quiet.
fn should_cargo_be_quiet() -> bool {
    !LogLevel::Report.is_enabled()
}

/// Run cargo check.
pub fn run_cargo_check() {
    report!("...Running `cargo check` to check the whole crate...");
    Command::new("cargo")
        .arg("check")
        .applied_to(|command| {
            if should_cargo_be_quiet() {
                command.arg("-q")
            } else {
                command
            }
        })
        .exec();
}

/// Run cargo check with -vv, returning stderr.
pub fn run_cargo_check_vv() -> String {
    report!("...Running `cargo check -vv` to obtain options...");
    let mut stderr = String::new();
    Command::new("cargo")
        .args(["check", "-vv"])
        .exec_with_stderr(&mut |line| {
            if LogLevel::Debug.is_enabled() {
                eprintln!("{line}");
            }
            stderr.push_str(&line);
            stderr.push('\n');
        });
    stderr
}

/// Mark crate dirty.
pub fn mark_crate_dirty() {
    report!("...Running `touch src/*.rs` to mark crate dirty...");
    Command::new("bash").args(["-c", "touch src/*.rs"]).exec();
}
