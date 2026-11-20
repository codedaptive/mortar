//! A report with its savings removed.

use crate::report::{OutValue, Report};

/// The report with every OutValue::Saved field removed; all else, the other fields' order included, unchanged.
pub fn without_savings(mut report: Report) -> Report {
    report.fields.retain(|(_, value)| !matches!(value, OutValue::Saved(_)));
    report
}
