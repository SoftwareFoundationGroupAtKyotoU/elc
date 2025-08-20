//! For the `run` command.

use crate::cargo::run_cargo_check;
use crate::cli::{RunArgs, TopArgs};
use crate::debug_println;
use crate::rustc_settings::{create_rustc_settings, load_rustc_settings};
use crate::util::exists_path;
use rustc_ast::Crate;
use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;

/// Argument passed to [`run_compiler`].
#[derive(Clone, Copy)]
struct Entry<'a> {
    /// Top-level arguments.
    top_args: &'a TopArgs,
}

/// Callbacks of [`run_compiler`].
impl Callbacks for Entry<'_> {
    fn after_crate_root_parsing(&mut self, _: &Compiler, _: &mut Crate) -> Compilation {
        println!("...Parsing succeeded...");
        Compilation::Continue
    }
    fn after_expansion(&mut self, _: &Compiler, _: TyCtxt) -> Compilation {
        println!("...Expansion succeeded...");
        Compilation::Continue
    }
    fn after_analysis(&mut self, _: &Compiler, tcx: TyCtxt) -> Compilation {
        println!("...Analysis succeeded!");
        run_body(*self, tcx);
        Compilation::Stop
    }
}

/// Perform the `run` command.
pub fn run(top_args: &TopArgs, run_args: &RunArgs) {
    println!("Initializing for running elcc...");
    let rustc_settings_path = &run_args.rustc_settings_path;
    run_cargo_check();
    if run_args.force_init || !exists_path(rustc_settings_path) {
        create_rustc_settings(top_args, rustc_settings_path);
    }
    let rustc_args = load_rustc_settings(top_args, rustc_settings_path, &run_args.rustc_args);
    debug_println!(top_args, "Arguments to rustc: {rustc_args:?}");
    println!("...Done!");
    println!("Running rustc...");
    run_compiler(&rustc_args, &mut Entry { top_args });
}

/// Body executed by `after_analysis`.
fn run_body(entry: Entry, tcx: TyCtxt) {
    println!("Running elcc...");
    let top_args = entry.top_args;
    println!("...Enumerating MIR keys...");
    for id in tcx.mir_keys(()) {
        let id = id.to_def_id();
        let path = tcx.def_path(id);
        debug_println!(top_args, "MIR key {path:?}");
        let path_str = tcx.def_path_str(id);
        println!("  {path_str}");
    }
    println!("...Not implemented yet, sorry!");
}
