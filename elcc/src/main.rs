//! Elc compiler based on rustc.

#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

mod cargo;
mod cli;
mod init;
mod run;
mod rustc_settings;
mod util;

use crate::cli::exec_cli;

/// The entry point.
fn main() {
    exec_cli();
}
