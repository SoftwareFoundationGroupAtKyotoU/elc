//! Utility.

use crate::error;
use std::fs;
use std::io::{BufRead as _, BufReader, Write as _, stdin, stdout};
use std::process::{Command, Stdio};
use std::time::SystemTime;

/// Result with an empty error.
pub type Result<T> = std::result::Result<T, ()>;

/// Turns [`bool`] to [`Result<()>`].
pub fn ok_err(b: bool) -> Result<()> {
    if b { Ok(()) } else { Err(()) }
}

/// Allows applying an arbitrary function
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

/// Reads a line from stdin, with error handling.
pub fn read_line() -> Result<String> {
    let mut line = String::new();
    stdin()
        .read_line(&mut line)
        .map_err(|err| error!("Error reading a line from stdin: {}", err))?;
    Ok(line)
}

/// Reads a line from stdin and returns the trimmed version, with error handling.
pub fn read_line_trim() -> Result<String> {
    Ok(read_line()?.trim().to_owned())
}

/// Flush a stdout, with error handling.
pub fn flush_stdout() -> Result<()> {
    stdout()
        .flush()
        .map_err(|err| error!("Could not flush stdout: {}", err))
}

/// Checks if a path exists, with error handling.
pub fn exists_path(path: &str) -> Result<bool> {
    fs::exists(path).map_err(|err| error!("Error in checking if `{path}` exists: {err}"))
}

/// Reads from a file, with error handling.
pub fn read_file(path: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|err| error!("Reading from {path} failed: {err}"))
}

/// Reads from a file as UTF-8, with error handling.
pub fn read_file_utf8(path: &str) -> Result<String> {
    String::from_utf8(read_file(path)?).map_err(|err| error!("Could not parse as utf8: {err}"))
}

/// Gets the file metadata, with error handling.
pub fn get_metadata(path: &str) -> Result<fs::Metadata> {
    fs::metadata(path).map_err(|err| error!("Could not get the metadata of `{path}`: {err}"))
}

/// Gets the file modification time, with error handling.
pub fn get_time_modified(path: &str) -> Result<SystemTime> {
    get_metadata(path)?
        .modified()
        .map_err(|err| error!("Getting the file modification time of `{path}` failed: {err}"))
}

/// Extend [`Command`] with methods.
pub trait CommandExtra {
    /// Executes a command, streaming stdout and stderr.
    fn exec(&mut self) -> Result<()>;
    /// Executes a command, streaming stdout but capturing stderr.
    fn exec_with_stderr<F: FnMut(String) -> ()>(
        &mut self,
        process_stderr_line: &mut F,
    ) -> Result<()>;
}

impl CommandExtra for Command {
    fn exec(&mut self) -> Result<()> {
        let exit_status = self
            .spawn()
            .map_err(|err| error!("Error in spawning: {err}"))?
            .wait()
            .map_err(|err| error!("Error in waiting: {err}"))?;
        ok_err(exit_status.success())
            .map_err(|_| error!("Failed with the exit_status {exit_status}"))
    }

    fn exec_with_stderr<F: FnMut(String) -> ()>(
        &mut self,
        process_stderr_line: &mut F,
    ) -> Result<()> {
        let mut child = self
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| error!("Error in spawning: {err}"))?;
        for line in BufReader::new(child.stderr.take().unwrap()).lines() {
            process_stderr_line(
                line.map_err(|err| error!("Failed to get a line from stderr: {err}"))?,
            );
        }
        let exit_status = child
            .wait()
            .map_err(|err| error!("Error in waiting: {err}"))?;
        ok_err(exit_status.success())
            .map_err(|_| error!("Failed with the exit_status {exit_status}"))
    }
}
