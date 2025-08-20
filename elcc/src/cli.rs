//! For the command-line interface.

use crate::log::{LogFilter, init_log_filter};
use crate::{debug, init, run};
use clap::{ArgAction, Args, Parser, Subcommand};
use std::io::{IsTerminal, stderr};

/// Command-line interface for [`clap`].
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Verbosity.
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// Choose plain non-ANSI logging.
    #[arg(long, global = true, default_value_t = !stderr().is_terminal())]
    pub plain: bool,
    /// Command.
    #[command(subcommand)]
    pub command: Command,
}

/// Verbosity.
#[derive(Args, Debug, Clone, Copy)]
pub struct Verbosity {
    /// Make elcc more verbose.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
    /// Make elcc more quiet.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub quiet: u8,
}

/// Default file path for rustc settings.
pub const DEFAULT_RUSTC_SETTINGS_PATH: &str = "target/debug/elcc-rustc-settings";

/// Subcommand for [`clap`].
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize.
    Init(InitArgs),
    /// Run the static verifier.
    Run(RunArgs),
}

/// Arguments for the `init` command.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force initialization of the rustc settings.
    #[arg(short, long)]
    pub force: bool,
    /// Path to the rustc settings.
    #[arg(long("rustc-settings"), default_value = DEFAULT_RUSTC_SETTINGS_PATH)]
    pub rustc_settings_path: String,
}

// Arguments for the `run` command.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Force initialization of the rustc settings.
    #[arg(long)]
    pub force_init: bool,
    /// Path to the rustc settings.
    #[arg(long("rustc-settings"), default_value = DEFAULT_RUSTC_SETTINGS_PATH)]
    pub rustc_settings_path: String,
    /// Arguments to rustc.
    #[arg(last = true)]
    pub rustc_args: Vec<String>,
}

/// Calculate [`LogFilter`] from the verbosity
fn calc_log_filter(verbosity: Verbosity) -> LogFilter {
    let Verbosity { verbose, quiet } = verbosity;
    assert!(
        verbose == 0 || quiet == 0,
        "Should not set both -v/--verbose and -q/-quiet",
    );
    let default = LogFilter::Info as i32;
    LogFilter::from(default + verbose as i32 - quiet as i32)
}

/// Parse and execute a `Cli`.
pub fn exec_cli() {
    let cli = Cli::parse();
    let log_filter = calc_log_filter(cli.verbosity);
    init_log_filter(log_filter);
    if !cli.plain {
        yansi::enable();
        debug!("Enabled ANSI output.");
    } else {
        yansi::disable();
        debug!("Disabled ANSI output.");
    }
    debug!("Cli argument: {cli:?}");
    debug!("Log filter: {log_filter:?}");
    match &cli.command {
        Command::Init(args) => init::init(args),
        Command::Run(args) => run::run(args),
    }
}
