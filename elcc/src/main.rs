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
    if cli.debug {
        println!("# Cli argument: {:?}", cli);
    }
    match &cli.command {
        Command::Init => init(&cli),
        Command::Run { rs_path, last_args } => run(&cli, rs_path, last_args),
    }
}
