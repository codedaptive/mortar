//! A report with its savings added.

use super::Savings;
use crate::report::{ExitCode, OutValue, Report};

/// The --json member name of the savings. Not "saved": mr status already has a "saved" boolean.
const SAVINGS_KEY: &str = "savings";

/// The report with ("savings", Saved(savings)) appended as its last field when it is Done; a refusal, conflict,
/// busy, usage error or failure comes back unchanged, so it carries no savings.
pub fn with_savings(mut report: Report, savings: Savings) -> Report {
    if report.exit == ExitCode::Done {
        report.fields.push((SAVINGS_KEY.to_string(), OutValue::Saved(savings)));
    }
    report
}
