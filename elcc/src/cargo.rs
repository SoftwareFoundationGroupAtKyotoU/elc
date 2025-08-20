//! For using cargo.

use crate::cli::TopArgs;
use crate::util::{exec_command, exec_command_with_stderr};
use std::process::Command;

/// Run cargo check.
pub fn run_cargo_check() {
    println!("...Running `cargo check` to check the whole crate...");
    exec_command(Command::new("cargo").arg("check"));
}

/// Run cargo check with -vv, returning stderr
pub fn run_cargo_check_vv(top_args: &TopArgs) -> String {
    println!("...Running `cargo check -vv` to obtain options...");
    let mut stderr = String::new();
    exec_command_with_stderr(Command::new("cargo").args(["check", "-vv"]), &mut |line| {
        if top_args.debug {
            eprintln!("{line}");
        }
        stderr.push_str(&line);
        stderr.push('\n');
    });
    stderr
}

/// Mark crate dirty.
pub fn mark_crate_dirty() {
    println!("...Running `touch src/*.rs` to mark crate dirty...");
    exec_command(Command::new("bash").args(["-c", "touch src/*.rs"]));
}
