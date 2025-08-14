use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;
use snailquote::unescape;
use std::env;
use std::fs;

use crate::base::{Cli, RUSTC_SETTINGS_PATH};
use crate::debug_println;
use crate::init::init;

/// Argument passed to [`run_compiler`]
struct Entry<'a> {
    /// Cli
    cli: &'a Cli,
}

/// Callbacks of [`run_compiler`]
impl Callbacks for Entry<'_> {
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        _krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        println!("...Success!");
        Compilation::Continue
    }
    fn after_analysis(&mut self, _: &Compiler, tcx: TyCtxt) -> Compilation {
        run_body(self.cli, tcx);
        Compilation::Stop
    }
}

/// Set up the environment variables and construct the arguments for `rustc`
fn rustc_setup(cli: &Cli, rs_path: &str, last_args: &Vec<String>) -> Vec<String> {
    let rustc_settings = String::from_utf8(fs::read(RUSTC_SETTINGS_PATH).unwrap_or_else(|err| {
        debug_println!(cli, "Reading from {} failed: {}", RUSTC_SETTINGS_PATH, err);
        init(cli, false);
        fs::read(RUSTC_SETTINGS_PATH)
            .unwrap_or_else(|err| panic!("Reading from {} failed: {}", RUSTC_SETTINGS_PATH, err))
    }))
    .unwrap_or_else(|err| panic!("Could not parse as utf8: {}", err));
    let mut rustc_settings = rustc_settings.lines();
    let rustc_options = rustc_settings.next_back().expect("No lines found!");
    rustc_settings.for_each(|line| {
        let idx = line
            .find("=")
            .unwrap_or_else(|| panic!("Could not found \"=\" in {}", line));
        let key = &line[..idx];
        let val =
            unescape(&line[idx + 1..]).unwrap_or_else(|err| panic!("Failed to unescape: {}", err));
        debug_println!(cli, "Set {} = {}", key, val);
        unsafe {
            env::set_var(key, val);
        }
    });
    let mut args = vec!["rustc".to_owned(), rs_path.to_owned()];
    rustc_options
        .split_ascii_whitespace()
        .for_each(|s| args.push(s.to_string()));
    last_args.iter().for_each(|arg| args.push(arg.clone()));
    args
}

/// Perform the `run` command
pub fn run(cli: &Cli, rs_path: &str, last_args: &Vec<String>) {
    let rustc_args = rustc_setup(cli, rs_path, last_args);
    debug_println!(cli, "Arguments to rustc: {:?}", rustc_args);
    println!("Running rustc...");
    run_compiler(&rustc_args, &mut Entry { cli: &cli });
}

/// Body executed by `after_analysis`
fn run_body(cli: &Cli, tcx: TyCtxt) {
    println!("Running elcc...");
    let _ = cli;
    let _ = tcx;
    println!("...Not implemented yet, sorry!");
}
