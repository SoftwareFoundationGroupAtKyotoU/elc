//! For using cargo.

use crate::log::LogLevel;
use crate::report;
use crate::util::{exec_command, exec_command_with_stderr};
use std::process::Command;

/// Check if cargo should be quiet
fn should_cargo_be_quiet() -> bool {
    !LogLevel::Report.is_enabled()
}

/// Run cargo check.
pub fn run_cargo_check() {
    report!("...Running `cargo check` to check the whole crate...");
    let mut command = Command::new("cargo");
    let mut command = command.arg("check");
    if should_cargo_be_quiet() {
        command = command.arg("-q");
    }
    exec_command(command);
}

/// Run cargo check with -vv, returning stderr
pub fn run_cargo_check_vv() -> String {
    report!("...Running `cargo check -vv` to obtain options...");
    let mut stderr = String::new();
    exec_command_with_stderr(Command::new("cargo").args(["check", "-vv"]), &mut |line| {
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
    exec_command(Command::new("bash").args(["-c", "touch src/*.rs"]));
}
