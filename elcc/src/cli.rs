//! For the command-line interface.

use crate::log::{LogFilter, LogFilterError, set_log_filter};
use crate::util::Result;
use crate::{debug, error, init, run};
use clap::{ArgAction, Args, CommandFactory as _, Parser, Subcommand};
use clap_complete::Shell;
use clap_complete::aot::{ValueHint, generate};
use std::io::{IsTerminal, stderr, stdout};

/// Command-line interface for [`clap`].
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_subcommand = true)]
pub struct Cli {
    /// Verbosity.
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// Suppress ANSI styling of logs.
    #[arg(long, global = true, default_value_t = !stderr().is_terminal())]
    pub no_ansi: bool,
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
pub fn detect_shell() -> Result<Shell> {
    let shell = Shell::from_env().ok_or_else(|| error!("Could not detect a supported shell"))?;
    debug!("Detected a supported shell: {shell}");
    Ok(shell)
}

/// Generates completion.
pub fn complete(args: &CompleteArgs) -> Result<()> {
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
    if !cli.no_ansi {
        yansi::enable();
    } else {
        yansi::disable();
    }
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
