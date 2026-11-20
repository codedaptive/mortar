//! The savings a report carries.

use super::Savings;
use crate::report::{OutValue, Report};

/// The value of the first field that is OutValue::Saved; None when no field is.
pub fn report_savings(report: &Report) -> Option<&Savings> {
    report.fields.iter().find_map(|(_, value)| match value {
        OutValue::Saved(savings) => Some(savings),
        _ => None,
    })
}
