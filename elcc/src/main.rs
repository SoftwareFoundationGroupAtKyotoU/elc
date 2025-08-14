#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;

mod base;
mod init;
mod run;

use crate::base::{Command, parse_cli};
use crate::init::init;
use crate::run::run;

fn main() {
    let cli = parse_cli();
    debug_println!(cli, "Cli argument: {:?}", cli);
    match &cli.command {
        Command::Init { force } => init(&cli, *force),
        Command::Run { last_args } => run(&cli, last_args),
    }
}
