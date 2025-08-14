use clap::{Parser, Subcommand};

/// Command-line interface for [`clap`]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Debug
    #[arg(long)]
    pub debug: bool,
    /// Verbose
    #[arg(short, long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}

/// Subcommand for [`clap`]
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize
    Init {
        /// Force initialization
        #[arg(short, long)]
        force: bool,
    },
    /// Run the static verifier
    Run {
        /// Path to the Rust source file
        rs_path: String,
        /// Arguments to Rust variables
        #[arg(last = true)]
        last_args: Vec<String>,
    },
}

/// Parse a `cli` function
pub fn parse_cli() -> Cli {
    let mut cli = Cli::parse();
    if cli.debug {
        cli.verbose = true;
    }
    cli
}

/// File path for rustc settings
pub const RUSTC_SETTINGS_PATH: &str = "target/debug/elcc-rustc-settings";

/// Print for debugging
#[macro_export]
macro_rules! debug_println {
    ($cli:expr, $fmt:expr $(, $args:expr)* $(,)?) => {
        if $cli.debug {
            println!(concat!("# ", $fmt) $(, $args)*);
        }
    };
}
