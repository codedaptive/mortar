//! formatter_argv: the arguments a formatter is run with, or None when nothing runs.

use crate::formatter::Formatter;
use std::ffi::OsString;
use std::path::PathBuf;

/// None when there is no formatter or no path to format: nothing runs. Else the formatter's `args`, then each path
/// in order. The program is not in it: it is what runs, not an argument.
pub(crate) fn formatter_argv(formatter: Option<&Formatter>, paths: &[PathBuf]) -> Option<Vec<OsString>> {
    let formatter = formatter.filter(|_| !paths.is_empty())?;
    Some(formatter.args.iter().cloned().chain(paths.iter().map(|p| p.clone().into_os_string())).collect())
}
