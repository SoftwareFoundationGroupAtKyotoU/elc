//! For the `init` command.

use crate::cargo::run_cargo_check;
use crate::cli::InitArgs;
use crate::rustc_settings::{create_rustc_settings, is_rustc_settings_old};
use crate::util::{Result, exists_path};
use crate::{info, report};

/// Performs the init command.
pub fn init(init_args: &InitArgs) -> Result<()> {
    report!("Initializing for elcc...");
    let rustc_settings_path = &init_args.rustc_settings_path;
    if !init_args.force
        && exists_path(rustc_settings_path)?
        && !is_rustc_settings_old(rustc_settings_path)?
    {
        info!(
            "The rustc settings file already exists at `{rustc_settings_path}` and seems up to date. Pass -f/--force to force re-initialization."
        );
        return Ok(());
    }
    run_cargo_check()?;
    create_rustc_settings(rustc_settings_path)?;
    report!("...Done!");
    Ok(())
}
