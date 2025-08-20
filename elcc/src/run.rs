//! For the `run` command.

use crate::cargo::run_cargo_check;
use crate::cli::RunArgs;
use crate::rustc_settings::{create_rustc_settings, load_rustc_settings};
use crate::util::exists_path;
use crate::{debug, info, report};
use rustc_ast::Crate;
use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;

/// Argument passed to [`run_compiler`].
#[derive(Clone, Copy)]
struct Entry {}

/// Callbacks of [`run_compiler`].
impl Callbacks for Entry {
    fn after_crate_root_parsing(&mut self, _: &Compiler, _: &mut Crate) -> Compilation {
        report!("...Parsing succeeded...");
        Compilation::Continue
    }
    fn after_expansion(&mut self, _: &Compiler, _: TyCtxt) -> Compilation {
        report!("...Expansion succeeded...");
        Compilation::Continue
    }
    fn after_analysis(&mut self, _: &Compiler, tcx: TyCtxt) -> Compilation {
        report!("...Analysis succeeded!");
        run_body(tcx);
        Compilation::Stop
    }
}

/// Perform the `run` command.
pub fn run(run_args: &RunArgs) {
    report!("Initializing for running elcc...");
    let rustc_settings_path = &run_args.rustc_settings_path;
    run_cargo_check();
    if run_args.force_init || !exists_path(rustc_settings_path) {
        create_rustc_settings(rustc_settings_path);
    }
    let rustc_args = load_rustc_settings(rustc_settings_path, &run_args.rustc_args);
    debug!("Arguments to rustc: {rustc_args:?}");
    report!("...Done!");
    report!("Running rustc...");
    run_compiler(&rustc_args, &mut Entry {});
}

/// Body executed by `after_analysis`.
fn run_body(tcx: TyCtxt) {
    report!("Running elcc...");
    info!("MIR keys:");
    for id in tcx.mir_keys(()) {
        let id = id.to_def_id();
        let path = tcx.def_path(id);
        debug!("MIR key {path:?}");
        let path_str = tcx.def_path_str(id);
        info!("  {path_str}");
    }
    report!("...Not implemented yet, sorry!");
}
