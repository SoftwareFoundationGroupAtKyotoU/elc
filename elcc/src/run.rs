//! For the `run` command.

use crate::cargo::run_cargo_check;
use crate::cli::RunArgs;
use crate::rustc_settings::{create_rustc_settings, is_rustc_settings_old, load_rustc_settings};
use crate::util::{exists_path, flush_stdout, read_line_trim};
use crate::{debug, info, report, warn};
use rustc_ast::Crate;
use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_hir::def_id::DefId;
use rustc_interface::interface::Compiler;
use rustc_middle::mir::pretty::{PrettyPrintMirOptions, write_mir_fn};
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::{SourceMap, get_source_map};
use rustc_span::{FileName, FileNameDisplayPreference, RealFileName, SourceFile, Span};
use std::io::stdout;
use std::sync::Arc;

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
    let should_create_rustc_settings = run_args.force_init
        || !exists_path(rustc_settings_path)
        || (is_rustc_settings_old(rustc_settings_path)
            && {
                report!(
                    "...Renewing the rustc settings file at `{rustc_settings_path}` because it has been out of date..."
                );
                true
            });
    if should_create_rustc_settings {
        create_rustc_settings(rustc_settings_path);
    }
    let rustc_args = load_rustc_settings(rustc_settings_path, &run_args.rustc_args);
    debug!("Arguments to rustc: {rustc_args:?}");
    report!("...Done!");
    report!("Running rustc...");
    run_compiler(&rustc_args, &mut Entry {});
}

/// Get a source file.
fn get_source_file(source_map: &SourceMap, path: &str) -> Option<Arc<SourceFile>> {
    source_map
        .get_source_file(&FileName::Real(RealFileName::LocalPath(path.into())))
        .or_else(|| {
            warn!("    Could not find a source file at `{path}`!");
            None
        })
}

/// Return a span of a source file.
fn source_file_span(source_file: &SourceFile) -> Span {
    Span::with_root_ctxt(source_file.start_pos, source_file.end_position())
}

/// Print MIR keys.
fn print_mir_keys<F: FnMut(DefId) -> bool>(tcx: TyCtxt, pred: &mut F) {
    for &id in tcx.mir_keys(()) {
        let id = id.to_def_id();
        if !pred(id) {
            return;
        }
        debug!("    {:?}", tcx.def_path(id));
        println!("      {}", tcx.def_path_str(id));
    }
}

/// Execute a query.
fn exec_query(tcx: TyCtxt, source_map: &SourceMap, input: &str) -> Option<()> {
    match input.split(' ').collect::<Vec<_>>().as_slice() {
        ["srcs"] => {
            for source_file in source_map.files().iter() {
                println!(
                    "      {}",
                    source_file
                        .name
                        .display(FileNameDisplayPreference::Remapped)
                );
            }
        }
        ["src", path] => {
            let source_file = get_source_file(&source_map, path)?;
            let src = source_file.src.clone().expect("Source not available!");
            println!("    Source:\n{}", src);
        }
        ["mir", name] => {
            let id = tcx
                .mir_keys(())
                .iter()
                .find(|&&id| &tcx.def_path_str(id) == name);
            let id = match id {
                None => {
                    warn!("Could not find an MIR key whose name is `{name}`");
                    return None;
                }
                Some(&id) => id,
            };
            write_mir_fn(
                tcx,
                tcx.optimized_mir(id),
                &mut |_, _| Ok(()),
                &mut stdout(),
                PrettyPrintMirOptions::from_cli(tcx),
            )
            .unwrap_or_else(|err| panic!("Error in printing MIR: {err}"));
        }
        ["mirs"] => {
            println!("    MIR keys of the crate:");
            print_mir_keys(tcx, &mut |_| true);
        }
        ["mirs", path] => {
            let file_span = source_file_span(get_source_file(&source_map, path)?.as_ref());
            println!("    MIR keys at `{path}`:");
            print_mir_keys(tcx, &mut |id| file_span.contains(tcx.def_span(id)));
        }
        ["quit"] => {
            info!("    OK, quitting now.");
            return Some(());
        }
        _ => warn!("    Unrecognized query: {input}"),
    }
    None
}

/// Body executed by [`after_analysis`](Callbacks::after_analysis).
fn run_body(tcx: TyCtxt) {
    report!("Running elcc...");
    let source_map = get_source_map().expect("Getting the source map failed");
    loop {
        print!("  Enter a query: ");
        flush_stdout();
        let input = read_line_trim();
        debug!("Input: {input}");
        let res = exec_query(tcx, &source_map, &input);
        if res.is_some() {
            return;
        }
    }
}
