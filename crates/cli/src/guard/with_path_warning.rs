//! with_path_warning: `mr guard install`'s installed report with the PATH warning as its last line. Only
//! install_outcome calls it, on an installed outcome, so `mr guard status`, refusals and failures never carry the
//! warning.

use crate::report::Report;

/// Some appends the warning to `report.lines`, so plain output prints it before `next:` and --json carries it in
/// lines; None returns the report unchanged.
pub fn with_path_warning(mut report: Report, warning: Option<String>) -> Report {
    report.lines.extend(warning);
    report
}
