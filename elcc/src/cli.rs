//! For the command-line interface.

use crate::ansi::{set_ansi_stderr, set_ansi_stdout};
use crate::log::{LogFilter, LogFilterError, set_log_filter};
use crate::util::Result;
use crate::{debug, error, init, run};
use clap::{ArgAction, Args, CommandFactory as _, Parser, Subcommand};
use clap_complete::Shell;
use clap_complete::aot::{ValueHint, generate};
use std::io::{IsTerminal as _, stderr, stdout};

/// Command-line interface for [`clap`].
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_subcommand = true)]
struct Cli {
    /// Verbosity.
    #[command(flatten)]
    verbosity: Verbosity,
    /// Force ANSI styling of stdout.
    #[arg(long, global = true, conflicts_with("no_ansi_out"))]
    force_ansi_out: bool,
    /// Suppress ANSI styling of stdout.
    #[arg(long, global = true)]
    no_ansi_out: bool,
    /// Force ANSI styling of stderr.
    #[arg(long, global = true, conflicts_with("no_ansi_err"))]
    force_ansi_err: bool,
    /// Suppress ANSI styling of stderr.
    #[arg(long, global = true)]
    no_ansi_err: bool,
    /// Command.
    #[command(subcommand)]
    command: Command,
}

/// Verbosity.
#[derive(Args, Debug, Clone, Copy)]
struct Verbosity {
    /// Make elcc more verbose.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,
    /// Make elcc more quiet.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    quiet: u8,
}

/// Default file path for rustc settings.
const DEFAULT_RUSTC_SETTINGS_PATH: &str = "target/debug/elcc-rustc-settings";

/// Subcommand for [`clap`].
#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize.
    Init(InitArgs),
    /// Run the static verifier.
    Run(RunArgs),
    /// Generate completion.
    Complete(CompleteArgs),
}

/// Arguments for the `init` command.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force initialization of the rustc settings.
    #[arg(short, long)]
    pub force: bool,
    /// Path to the rustc settings.
    #[arg(long("rustc-settings"), value_hint = ValueHint::FilePath, default_value = DEFAULT_RUSTC_SETTINGS_PATH)]
    pub rustc_settings_path: String,
}

/// Arguments for the `run` command.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Force initialization of the rustc settings.
    #[arg(long, conflicts_with("no_init"))]
    pub force_init: bool,
    /// Suppress initialization of the rustc settings.
    #[arg(long)]
    pub no_init: bool,
    /// Path to the rustc settings.
    #[arg(long("rustc-settings"), value_hint = ValueHint::FilePath, default_value = DEFAULT_RUSTC_SETTINGS_PATH)]
    pub rustc_settings_path: String,
    /// Arguments to rustc.
    #[arg(last = true)]
    pub rustc_args: Vec<String>,
}

/// Arguments for the `complete` command.
#[derive(Args, Debug)]
pub struct CompleteArgs {
    /// Target shell, defaults to the current shell.
    #[arg(long)]
    pub shell: Option<Shell>,
    /// Name of the command.
    #[arg(long, default_value = "elcc")]
    pub name: String,
}

/// Calculates [`LogFilter`] from the verbosity.
fn calc_log_filter(verbosity: Verbosity) -> Result<LogFilter> {
    let Verbosity { verbose, quiet } = verbosity;
    if verbose != 0 && quiet != 0 {
        error!("Should not set both -v/--verbose and -q/--quiet.");
        return Err(());
    }
    LogFilter::try_from(LogFilter::default() as i8 + verbose as i8 - quiet as i8).map_err(|err| {
        match err {
            LogFilterError::TooVerbose => error!("Too many -v/--verbose ({verbose}) is specified!"),
            LogFilterError::TooQuiet => error!("Too many -q/--quiet ({quiet}) is specified!"),
        }
    })
}

/// Detects the shell.
fn detect_shell() -> Result<Shell> {
    let shell = Shell::from_env().ok_or_else(|| error!("Could not detect a supported shell"))?;
    debug!("Detected a supported shell: {shell}");
    Ok(shell)
}

/// Generates completion.
fn complete(args: &CompleteArgs) -> Result<()> {
    let shell = match args.shell {
        Some(shell) => shell,
        None => detect_shell()?,
    };
    generate(shell, &mut Cli::command(), &args.name, &mut stdout());
    Ok(())
}

/// Parses and executes a `Cli`.
pub fn exec_cli() -> Result<()> {
    let cli = Cli::parse();
    set_ansi_stdout(cli.force_ansi_out || !cli.no_ansi_out && stdout().is_terminal());
    set_ansi_stderr(cli.force_ansi_err || !cli.no_ansi_err && stderr().is_terminal());
    let log_filter = calc_log_filter(cli.verbosity)?;
    set_log_filter(log_filter);
    debug!("Log filter: {log_filter:?}");
    debug!("Cli argument: {cli:?}");
    match &cli.command {
        Command::Init(args) => init::init(args),
        Command::Run(args) => run::run(args),
        Command::Complete(args) => complete(args),
    }
}
