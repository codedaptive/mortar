//! install_outcome: the report `mr guard install` prints, with the PATH warning only on an installed
//! outcome. Pure.

use super::with_path_warning;
use crate::report::Report;

/// Ok (installed or already-installed) goes through with_path_warning, so the warning is its last line, printed
/// before `next:`. Err (a refusal, a failure or a busy lock) comes back unchanged, without the warning.
pub fn install_outcome(installed: Result<Report, Report>, warning: Option<String>) -> Report {
    match installed {
        Ok(report) => with_path_warning(report, warning),
        Err(report) => report,
    }
}
