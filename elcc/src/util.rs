//! Utility.

use crate::debug;
use std::fs;
use std::io::{BufRead as _, BufReader, Write as _, stdin, stdout};
use std::process::{Command, Stdio};
use std::time::SystemTime;

/// Allow applying an arbitrary function
/// in the form of the method [`applied_to`](AppliedTo::applied_to).
pub trait AppliedTo
where
    Self: Sized,
{
    /// Method for applying a function.
    #[inline(always)]
    fn applied_to<R, F: FnOnce(Self) -> R>(self, f: F) -> R {
        f(self)
    }
}
impl<T> AppliedTo for T {}

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

/// Extend [`Command`] with methods.
pub trait CommandExtra {
    /// Execute a command, streaming stdout and stderr.
    fn exec(&mut self);
    /// Execute a command, streaming stdout but capturing stderr.
    fn exec_with_stderr<F: FnMut(String) -> ()>(&mut self, process_stderr_line: &mut F);
}

impl CommandExtra for Command {
    fn exec(&mut self) {
        let exit_status = self
            .spawn()
            .unwrap_or_else(|err| panic!("Error in spawning: {err}"))
            .wait()
            .unwrap_or_else(|err| panic!("Error in waiting: {err}"));
        if !exit_status.success() {
            panic!("Failed with the exit_status {exit_status}")
        }
    }

    fn exec_with_stderr<F: FnMut(String) -> ()>(&mut self, process_stderr_line: &mut F) {
        let mut child = self
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
}
