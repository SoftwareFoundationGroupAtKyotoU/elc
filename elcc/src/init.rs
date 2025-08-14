use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::base::{Cli, RUSTC_SETTINGS_PATH};
use crate::debug_println;

/// Execute a command, streaming stdout and stderr
fn exec_command(command: &mut Command) {
    let exit_status = command
        .spawn()
        .unwrap_or_else(|err| panic!("Error in spawning: {}", err))
        .wait()
        .unwrap_or_else(|err| panic!("Error in waiting {}", err));
    if !exit_status.success() {
        panic!("Failed with the exit_status {}", exit_status)
    }
}

/// Execute a command, streaming stdout but capturing stderr
fn exec_command_with_stderr<F: FnMut(String) -> ()>(
    command: &mut Command,
    process_stderr_line: &mut F,
) {
    let mut child = command
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("Error in spawning: {}", err));
    BufReader::new(child.stderr.take().unwrap())
        .lines()
        .for_each(|line| {
            process_stderr_line(
                line.unwrap_or_else(|err| panic!("Failed to get a line from stderr: {}", err)),
            );
        });
    let exit_status = child
        .wait()
        .unwrap_or_else(|err| panic!("Error in waiting {}", err));
    if !exit_status.success() {
        panic!("Failed with the exit_status {}", exit_status)
    }
}

/// Modify rustc options
fn modify_rustc_options(rustc_options: String) -> String {
    lazy_static! {
        static ref JSON_REGEX: Regex = Regex::new("--json=\\S* ").unwrap();
    }
    // Disable json output
    let rustc_options = rustc_options.replace("--error-format=json ", "");
    let rustc_options = JSON_REGEX.replace_all(&rustc_options, "");
    // Hacky replacement: A workaround
    rustc_options.replace(", ", ",").replace("'", "")
}

/// Get rustc settings
fn get_settings(cli: &Cli) -> String {
    println!("...Running `cargo check -vv` to obtain options...");
    lazy_static! {
        static ref STDERR_REGEX: Regex =
            Regex::new("\\n     Running `((?:.|\\n)+) (\\S*?rustc) (.+?)`\\n").unwrap();
        static ref ARGS_REGEX: Regex = Regex::new("^(.*?) src/(.*?)\\.rs (.*?)$").unwrap();
    }
    let mut stderr = String::new();
    exec_command_with_stderr(Command::new("cargo").args(["check", "-vv"]), &mut |line| {
        if cli.debug {
            eprintln!("{}", line);
        }
        stderr.push_str(&line);
        stderr.push('\n');
    });
    let (_, [rustc_env, rustc_name, rustc_args]) = STDERR_REGEX
        .captures(&stderr)
        .unwrap_or_else(|| panic!("Could not find a rustc command in:\n{}", stderr))
        .extract();
    debug_println!(
        cli,
        "Found a rustc command: \n#  {}\n#  {}\n#  {}",
        rustc_env,
        rustc_name,
        rustc_args
    );
    debug_println!(cli, "Recorded env:\n{}", rustc_env);
    let (_, [prefix, rs_path, postfix]) = ARGS_REGEX
        .captures(rustc_args)
        .unwrap_or_else(|| panic!("Failed to parse rustc arguments `{}`", rustc_args))
        .extract();
    debug_println!(
        cli,
        "Parsed command:\n#   {}\n#   {}\n#   {}",
        prefix,
        rs_path,
        postfix
    );
    let rustc_options = modify_rustc_options(format!("{} {}", prefix, postfix));
    debug_println!(cli, "Recorded options:\n{}", rustc_options);
    assert!(
        rustc_options.chars().all(|c| c != '\n'),
        "Options contains a new line: {}",
        rustc_options
    );
    format!("{}\n{}", rustc_options, rustc_env)
}

/// Perform the init command
pub fn init(cli: &Cli, force: bool) {
    if !force {
        if fs::exists(RUSTC_SETTINGS_PATH).unwrap_or_else(|err| {
            panic!(
                "Error in checking if `{}` exists: {}",
                RUSTC_SETTINGS_PATH, err
            )
        }) {
            println!("Already initialized. Pass -f or --force to force re-initialization.");
            return;
        }
    }
    println!("Initializing for elcc...");
    println!("...Running `cargo check` to check the whole crate...");
    exec_command(Command::new("cargo").arg("check"));
    println!("...Running `touch src/*.rs` to mark crate dirty...");
    exec_command(Command::new("bash").args(["-c", "touch src/*.rs"]));
    let rustc_settings = get_settings(cli);
    println!(
        "...Saving the rustc options to `{}`...",
        RUSTC_SETTINGS_PATH
    );
    fs::write(RUSTC_SETTINGS_PATH, rustc_settings)
        .unwrap_or_else(|err| panic!("Could not write the rustc options: {}", err));
    println!("...Done!");
}
