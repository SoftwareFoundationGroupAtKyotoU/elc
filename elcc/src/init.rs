//! For the `init` command.

use crate::cargo::run_cargo_check;
use crate::cli::{InitArgs, TopArgs};
use crate::rustc_settings::create_rustc_settings;
use crate::util::exists_path;

/// Perform the init command.
pub fn init(top_args: &TopArgs, init_args: &InitArgs) {
    println!("Initializing for elcc...");
    let rustc_settings_path = &init_args.rustc_settings_path;
    if !init_args.force && exists_path(rustc_settings_path) {
        println!(
            "...Rustc settings already exists at `{rustc_settings_path}`. Pass -f or --force to force re-initialization."
        );
        return;
    }
    run_cargo_check();
    create_rustc_settings(top_args, rustc_settings_path);
    println!("...Done!");
}
