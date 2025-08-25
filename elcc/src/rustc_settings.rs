//! For the rustc settings.

use crate::cargo::{mark_crate_dirty, run_cargo_check_vv};
use crate::util::{get_time_modified, read_file_utf8};
use crate::{debug, report};
use lazy_static::lazy_static;
use regex::Regex;
use std::{env, fs};

/// Checks if the rustc settings file is out of date.
pub fn is_rustc_settings_old(rustc_settings_path: &str) -> bool {
    let base_time = match get_time_modified(rustc_settings_path) {
        None => return false,
        Some(base_time) => base_time,
    };
    let res = match get_time_modified("Cargo.toml") {
        None => false,
        Some(cargo_toml_time) => base_time < cargo_toml_time,
    } || match get_time_modified("Cargo.lock") {
        None => false,
        Some(cargo_lock_time) => base_time < cargo_lock_time,
    };
    res
}

/// Separator between the environment arguments and options.
const RUSTC_SETTINGS_SEP: &str = "\n[RUSTC]\n";

/// Text explanation of [`RUSTC_SETTINGS_SEP`].
const RUSTC_SETTINGS_SEP_TEXT: &str = "\"\\n[RUSTC]\\n\"";

/// Modifies rustc arguments.
fn modify_rustc_args(rustc_args: String) -> String {
    lazy_static! {
        static ref JSON_REGEX: Regex = Regex::new("--json=\\S* ").unwrap();
    }
    // Disables json output
    let rustc_options = rustc_args.replace("--error-format=json ", "");
    let rustc_options = JSON_REGEX.replace_all(&rustc_options, "");
    // Hacky replacement: A workaround
    rustc_options.replace(", ", ",").replace("'", "")
}

/// Gets rustc settings.
fn get_rustc_settings() -> String {
    let stderr = run_cargo_check_vv();
    lazy_static! {
        static ref STDERR_REGEX: Regex =
            Regex::new("\\n     Running `((?:.|\\n)+) (\\S*?rustc) (.+?)`\\n").unwrap();
    }
    let (_, [rustc_env, rustc_name, rustc_args]) = STDERR_REGEX
        .captures(&stderr)
        .unwrap_or_else(|| panic!("Could not find a rustc command in:\n{stderr}"))
        .extract();
    debug!("Found a rustc command:");
    debug!("  Environment: {rustc_env}");
    debug!("  Rustc: {rustc_name}");
    debug!("  Arguments: {rustc_args}");
    assert!(
        !rustc_args.contains(RUSTC_SETTINGS_SEP),
        "Error: {rustc_args} contains {RUSTC_SETTINGS_SEP_TEXT}"
    );
    let rustc_args = modify_rustc_args(rustc_args.to_owned());
    debug!("Modified arguments: {rustc_args}");
    format!("{rustc_env}{RUSTC_SETTINGS_SEP}{rustc_args}")
}

/// Creates rustc settings.
pub fn create_rustc_settings(rustc_settings_path: &str) {
    mark_crate_dirty();
    let rustc_settings = get_rustc_settings();
    report!("...Saving the rustc settings to `{rustc_settings_path}`...");
    fs::write(rustc_settings_path, rustc_settings)
        .unwrap_or_else(|err| panic!("Could not write the rustc settings: {err}"));
}

/// Processes environment variables.
fn process_env(mut env: &str) {
    debug!("Parsing environment variables: {env}");
    loop {
        let (key, env_) = env
            .split_once('=')
            .unwrap_or_else(|| panic!("Cannot find '=' in {env}"));
        env = env_;
        key.chars().for_each(|c| {
            assert!(
                c.is_ascii_alphanumeric() || c == '_',
                "Parsed key contains a character '{c}' not alphanumeric or '_': {key}"
            )
        });
        let mut val = String::new();
        if env.starts_with('\'') {
            env = &env[1..];
            loop {
                let (val_, env_) = env
                    .split_once('\'')
                    .unwrap_or_else(|| panic!("Could not find a closing quote in {env}"));
                val.push_str(val_);
                env = env_;
                if env.starts_with(' ') || env.is_empty() {
                    break;
                }
                assert!(env.starts_with("\\''"), "Not starting with \"\\''\": {env}");
                val.push('\'');
                env = &env[3..];
            }
        } else {
            let close_idx = env.find(' ').unwrap_or(env.len());
            val = env[..close_idx].to_owned();
            env = &env[close_idx..];
        };
        debug!("Set {key}={val}");
        unsafe {
            env::set_var(key, val);
        }
        if env.is_empty() {
            break;
        }
        assert!(
            env.chars().nth(0) == Some(' '),
            "Key-value pair not ending with ' '"
        );
        env = &env[1..];
    }
}

/// Loads rustc settings, sets environment variables and constructs the arguments for `rustc`.
pub fn load_rustc_settings(rustc_settings_path: &str, last_args: &Vec<String>) -> Vec<String> {
    report!("...Loading the rustc settings from `{rustc_settings_path}`...");
    let rustc_settings = read_file_utf8(rustc_settings_path);
    let (rustc_env, rustc_options) = rustc_settings
        .rsplit_once(RUSTC_SETTINGS_SEP)
        .unwrap_or_else(|| {
            panic!("Could not find {RUSTC_SETTINGS_SEP_TEXT} in rustc settings: {rustc_settings}")
        });
    process_env(rustc_env);
    let mut args = vec!["rustc".to_owned()];
    rustc_options
        .split_ascii_whitespace()
        .for_each(|s| args.push(s.to_string()));
    last_args.iter().for_each(|arg| args.push(arg.clone()));
    args
}
