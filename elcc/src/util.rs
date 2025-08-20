//! Utility.

use crate::debug;
use std::fs;
use std::io::{BufRead, BufReader, Write, stdin, stdout};
use std::process::{Command, Stdio};
use std::time::SystemTime;

/// Read a line from stdin, with error handling.
pub fn read_line() -> String {
    let mut line = String::new();
    stdin()
        .read_line(&mut line)
        .unwrap_or_else(|err| panic!("Error reading a line from stdin: {}", err));
    line
}

/// Read a line from stdin and trim, with error handling.
pub fn read_line_trim() -> String {
    read_line().trim().to_owned()
}

/// Flush a stdout, with error handling.
pub fn flush_stdout() {
    stdout()
        .flush()
        .unwrap_or_else(|err| panic!("Could not flush stdout: {}", err));
}

/// Check if a path exists, with error handling.
pub fn exists_path(path: &str) -> bool {
    fs::exists(path).unwrap_or_else(|err| panic!("Error in checking if `{path}` exists: {err}"))
}

/// Read from a file, with error handling.
pub fn read_file(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| {
        panic!("Reading from {path} failed: {err}");
    })
}

/// Read from a file as UTF-8, with error handling.
pub fn read_file_utf8(path: &str) -> String {
    String::from_utf8(read_file(path))
        .unwrap_or_else(|err| panic!("Could not parse as utf8: {err}"))
}

/// Get the file metadata, with error handling.
pub fn get_metadata(path: &str) -> Option<fs::Metadata> {
    match fs::metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(err) => {
            debug!("Could not get the metadata of `{path}`: {err}");
            None
        }
    }
}

/// Get the file modification time, with error handling.
pub fn get_time_modified(path: &str) -> Option<SystemTime> {
    match get_metadata(path)?.modified() {
        Ok(time) => Some(time),
        Err(err) => {
            debug!("Getting the file modification time of `{path}` failed: {err}");
            None
        }
    }
}

/// Execute a command, streaming stdout and stderr.
pub fn exec_command(command: &mut Command) {
    let exit_status = command
        .spawn()
        .unwrap_or_else(|err| panic!("Error in spawning: {err}"))
        .wait()
        .unwrap_or_else(|err| panic!("Error in waiting: {err}"));
    if !exit_status.success() {
        panic!("Failed with the exit_status {exit_status}")
    }
}

/// Execute a command, streaming stdout but capturing stderr.
pub fn exec_command_with_stderr<F: FnMut(String) -> ()>(
    command: &mut Command,
    process_stderr_line: &mut F,
) {
    let mut child = command
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("Error in spawning: {err}"));
    BufReader::new(child.stderr.take().unwrap())
        .lines()
        .for_each(|line| {
            process_stderr_line(
                line.unwrap_or_else(|err| panic!("Failed to get a line from stderr: {err}")),
            );
        });
    let exit_status = child
        .wait()
        .unwrap_or_else(|err| panic!("Error in waiting: {err}"));
    if !exit_status.success() {
        panic!("Failed with the exit_status {exit_status}")
    }
}
