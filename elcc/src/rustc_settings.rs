//! For the rustc settings.

use crate::cargo::{mark_crate_dirty, run_cargo_check_vv};
use crate::util::{get_time_modified, read_file_utf8};
use crate::{debug, report};
use lazy_static::lazy_static;
use regex::Regex;
use std::{env, fs};

/// Check if the rustc settings file is out of date.
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

/// Modify rustc arguments.
fn modify_rustc_args(rustc_args: String) -> String {
    lazy_static! {
        static ref JSON_REGEX: Regex = Regex::new("--json=\\S* ").unwrap();
    }
    // Disable json output
    let rustc_options = rustc_args.replace("--error-format=json ", "");
    let rustc_options = JSON_REGEX.replace_all(&rustc_options, "");
    // Hacky replacement: A workaround
    rustc_options.replace(", ", ",").replace("'", "")
}

/// Get rustc settings.
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
        "Error: {rustc_env} contains {RUSTC_SETTINGS_SEP_TEXT}"
    );
    let rustc_args = modify_rustc_args(rustc_args.to_owned());
    debug!("Modified arguments: {rustc_args}");
    format!("{rustc_env}{RUSTC_SETTINGS_SEP}{rustc_args}")
}

/// Create rustc settings.
pub fn create_rustc_settings(rustc_settings_path: &str) {
    mark_crate_dirty();
    let rustc_settings = get_rustc_settings();
    report!("...Saving the rustc settings to `{rustc_settings_path}`...");
    fs::write(rustc_settings_path, rustc_settings)
        .unwrap_or_else(|err| panic!("Could not write the rustc settings: {err}"));
}

/// Process environment variables.
fn process_env(mut env: &str) {
    debug!("Parsing environment variables: {env}");
    loop {
        let eq_idx = env
            .find('=')
            .unwrap_or_else(|| panic!("Cannot find '=' in {env}"));
        let key = &env[..eq_idx];
        key.chars().for_each(|c| {
            assert!(
                c.is_ascii_alphanumeric() || c == '_',
                "Parsed key contains a character '{c}' not alphanumeric or '_': {key}"
            )
        });
        env = &env[eq_idx + 1..];
        let mut val = String::new();
        if env.starts_with('\'') {
            env = &env[1..];
            loop {
                let close_idx = env
                    .find('\'')
                    .unwrap_or_else(|| panic!("Could not find a closing quote in {env}"));
                val.push_str(&env[..close_idx]);
                env = &env[close_idx + 1..];
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

/// Load rustc settings, set environment variables and construct the arguments for `rustc`.
pub fn load_rustc_settings(rustc_settings_path: &str, last_args: &Vec<String>) -> Vec<String> {
    report!("...Loading the rustc settings from `{rustc_settings_path}`...");
    let rustc_settings = read_file_utf8(rustc_settings_path);
    let sep_idx = rustc_settings.find(RUSTC_SETTINGS_SEP).unwrap_or_else(|| {
        panic!("Could not find {RUSTC_SETTINGS_SEP_TEXT} in rustc settings: {rustc_settings}")
    });
    let rustc_env = &rustc_settings[0..sep_idx];
    let rustc_options = &rustc_settings[sep_idx + RUSTC_SETTINGS_SEP.len()..];
    process_env(rustc_env);
    let mut args = vec!["rustc".to_owned()];
    rustc_options
        .split_ascii_whitespace()
        .for_each(|s| args.push(s.to_string()));
    last_args.iter().for_each(|arg| args.push(arg.clone()));
    args
}
