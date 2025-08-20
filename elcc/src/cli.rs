use clap::{Args, Parser, Subcommand};

use crate::{init::init, run::run};

/// Command-line interface for [`clap`].
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Top-level arguments.
    #[command(flatten)]
    pub args: TopArgs,
    /// Command.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level arguments.
#[derive(Args, Debug)]
pub struct TopArgs {
    /// Log for debugging.
    #[arg(long, global = true)]
    pub debug: bool,
    /// Be verbose.
    #[arg(short, long, global = true)]
    pub verbose: bool,
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

/// Print for debugging.
#[macro_export]
macro_rules! debug_println {
    ($top_args:expr, $fmt:expr $(, $args:expr)* $(,)?) => {
        if $top_args.debug {
            print!("# ");
            println!($fmt $(, $args)*);
        }
    };
}

/// Parse and execute a `Cli`.
pub fn exec_cli() {
    let cli = Cli::parse();
    let top_args = &cli.args;
    debug_println!(top_args, "Cli argument: {cli:?}");
    match &cli.command {
        Command::Init(args) => init(top_args, args),
        Command::Run(args) => run(top_args, args),
    }
}
