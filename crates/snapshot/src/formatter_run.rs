//! FormatterRun: one formatter run a snapshot made, and how it ended. A declaration only; snapshot_from_apply_tests
//! carries one, run_formatter makes it, and the integration cases run_formatter's header lists exercise it.

use crate::formatter_result::FormatterResult;
use std::ffi::OsString;
use std::path::PathBuf;

/// The formatter's `program` and `args` as the caller gave them (without the paths), the `paths` it was given, in
/// the order passed and relative to the folder, and its `result`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatterRun {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub paths: Vec<PathBuf>,
    pub result: FormatterResult,
}
