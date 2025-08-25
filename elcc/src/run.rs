//! For the `run` command.

use crate::cargo::run_cargo_check;
use crate::cli::RunArgs;
use crate::rustc_settings::{create_rustc_settings, is_rustc_settings_old, load_rustc_settings};
use crate::util::{exists_path, flush_stdout, read_line_trim};
use crate::{debug, info, report, warn};
use rustc_ast::Crate;
use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface::Compiler;
use rustc_middle::mir::pretty::{PrettyPrintMirOptions, write_mir_fn};
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::get_source_map;
use rustc_span::{FileName, RealFileName, Span};
use std::io::stdout;

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

/// Body executed by [`after_analysis`](Callbacks::after_analysis).
fn run_body(tcx: TyCtxt) {
    report!("Running elcc...");
    let source_map = get_source_map().expect("Getting the source map failed");
    loop {
        print!("  Enter a source file path: ");
        flush_stdout();
        let source_file_path = read_line_trim();
        debug!("Input: {source_file_path}");
        if &source_file_path == "quit" {
            info!("    Ok, quitting now.");
            return;
        }
        let source_file = source_map.get_source_file(&FileName::Real(RealFileName::LocalPath(
            (&source_file_path).into(),
        )));
        let source_file = match source_file {
            None => {
                warn!("    Could not find a source file at `{source_file_path}`!");
                continue;
            }
            Some(source_file) => source_file,
        };
        let start_pos = source_file.start_pos;
        let end_pos = source_file.end_position();
        debug!("Position of the file: {start_pos:?} - {end_pos:?}");
        let file_span = Span::with_root_ctxt(start_pos, end_pos);
        debug!("Span for the file: {:?}", file_span);
        loop {
            print!("  Enter what you want: ");
            flush_stdout();
            let input = read_line_trim();
            debug!("Input: {input}");
            let query = input.split(' ').collect::<Vec<_>>();
            match query.as_slice() {
                ["src"] => {
                    let src = source_file.src.clone().expect("Source not available!");
                    println!("    Source:\n{}", src);
                }
                ["mir", name] => {
                    let (id, path_str) = tcx
                        .mir_keys(())
                        .iter()
                        .filter(|&&id| file_span.contains(tcx.def_span(id)))
                        .find_map(|&id| {
                            let path_str = tcx.def_path_str(id);
                            path_str.contains(name).then_some((id, path_str))
                        })
                        .unwrap_or_else(|| {
                            panic!("Could not find an MIR key whose name contains `{name}`")
                        });
                    report!("Printing the MIR of `{}`:", path_str);
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
                    println!("    MIR keys:");
                    for id in tcx.mir_keys(()) {
                        let id = id.to_def_id();
                        let path = tcx.def_path(id);
                        debug!("MIR key {path:?}");
                        let path_str = tcx.def_path_str(id);
                        let span = tcx.def_span(id);
                        debug!("Span: {span:?}");
                        if !file_span.contains(span) {
                            debug!("Not in the file");
                            continue;
                        }
                        println!("      {path_str}");
                    }
                }
                ["done"] => {
                    info!("    OK.");
                    break;
                }
                ["quit"] => {
                    info!("    OK, quitting now.");
                    return;
                }
                _ => {
                    warn!("    Unrecognized query: {input}");
                    continue;
                }
            }
        }
    }
}
