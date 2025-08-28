//! Elc compiler based on rustc.

#![feature(rustc_private)]
#![feature(bool_to_result)]
#![macro_use]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

mod ansi;
mod cargo;
mod cli;
mod init;
mod log;
mod run;
mod rustc_settings;
mod util;

use crate::cli::exec_cli;

/// The entry point.
fn main() {
    let _ = exec_cli();
}
