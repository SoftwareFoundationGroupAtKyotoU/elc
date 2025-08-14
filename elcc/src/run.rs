use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;
use std::env;
use std::fs;

use crate::base::{Cli, RUSTC_SETTINGS_PATH};
use crate::debug_println;
use crate::init::init;

/// Argument passed to [`run_compiler`]
struct Entry<'a> {
    /// Cli
    cli: &'a Cli,
}

/// Callbacks of [`run_compiler`]
impl Callbacks for Entry<'_> {
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        _krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        println!("...Success!");
        Compilation::Continue
    }
    fn after_analysis(&mut self, _: &Compiler, tcx: TyCtxt) -> Compilation {
        run_body(self.cli, tcx);
        Compilation::Stop
    }
}

/// Process environment variables
fn process_env(cli: &Cli, mut env: &str) {
    debug_println!(cli, "Parsing environment variables: {}", env);
    loop {
        let eq_idx = env
            .find('=')
            .unwrap_or_else(|| panic!("Cannot find '=' in {}", env));
        let key = &env[..eq_idx];
        key.chars().for_each(|c| {
            assert!(
                c.is_ascii_alphanumeric() || c == '_',
                "Parsed key contains a character '{ }' not alphanumeric or '_': {}",
                c,
                key
            )
        });
        env = &env[eq_idx + 1..];
        let mut val = String::new();
        if env.starts_with('\'') {
            env = &env[1..];
            loop {
                let close_idx = env
                    .find('\'')
                    .unwrap_or_else(|| panic!("Could not find a closing quote in {}", env));
                val.push_str(&env[..close_idx]);
                env = &env[close_idx + 1..];
                if env.starts_with(' ') || env.is_empty() {
                    break;
                }
                assert!(
                    env.starts_with("\\''"),
                    "Not starting with \"\\''\": {}",
                    env
                );
                val.push('\'');
                env = &env[3..];
            }
        } else {
            let close_idx = env.find(' ').unwrap_or(env.len());
            val = env[..close_idx].to_owned();
            env = &env[close_idx..];
        };
        debug_println!(cli, "Set {}={}", key, val);
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

/// Set up the environment variables and construct the arguments for `rustc`
fn rustc_setup(cli: &Cli, last_args: &Vec<String>) -> Vec<String> {
    let rustc_settings = String::from_utf8(fs::read(RUSTC_SETTINGS_PATH).unwrap_or_else(|err| {
        debug_println!(cli, "Reading from {} failed: {}", RUSTC_SETTINGS_PATH, err);
        init(cli, false);
        fs::read(RUSTC_SETTINGS_PATH)
            .unwrap_or_else(|err| panic!("Reading from {} failed: {}", RUSTC_SETTINGS_PATH, err))
    }))
    .unwrap_or_else(|err| panic!("Could not parse as utf8: {}", err));
    let sep_idx = rustc_settings.find('\n').unwrap_or_else(|| {
        panic!(
            "Could not find a new line in rustc settings: {}",
            rustc_settings
        )
    });
    let rustc_options = &rustc_settings[0..sep_idx];
    let rustc_env = &rustc_settings[sep_idx + 1..];
    process_env(cli, rustc_env);
    let mut args = vec!["rustc".to_owned()];
    rustc_options
        .split_ascii_whitespace()
        .for_each(|s| args.push(s.to_string()));
    last_args.iter().for_each(|arg| args.push(arg.clone()));
    args
}

/// Perform the `run` command
pub fn run(cli: &Cli, last_args: &Vec<String>, force_init: bool) {
    if force_init {
        init(cli, true);
    }
    let rustc_args = rustc_setup(cli, last_args);
    debug_println!(cli, "Arguments to rustc: {:?}", rustc_args);
    println!("Running rustc...");
    run_compiler(&rustc_args, &mut Entry { cli: &cli });
}

/// Body executed by `after_analysis`
fn run_body(cli: &Cli, tcx: TyCtxt) {
    println!("Running elcc...");
    println!("...Enumerating MIR keys...");
    for id in tcx.mir_keys(()) {
        let id = id.to_def_id();
        let path = tcx.def_path(id);
        debug_println!(cli, "MIR key {:?}", path);
        let path_str = tcx.def_path_str(id);
        println!("  {}", path_str);
    }
    println!("...Not implemented yet, sorry!");
}
