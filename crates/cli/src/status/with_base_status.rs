//! with_base_status: a request folder's status report with what it stands on: the record's base, its partners and
//! whether a partner moved since.

use super::BaseStatus;
use crate::report::{OutValue, Report};
use crate::request_base::{partners_json, partners_line};

/// `report` with ("base", its hex), ("partners", partners_json) and ("partner_moved", true when any partner moved)
/// after its own fields, for every request folder. Its lines gain "base <base>", partners_line and, when a partner
/// moved, "partner zones moved since the base: <zones>" only when the request has partners, so a plain request's
/// lines are unchanged.
pub fn with_base_status(mut report: Report, status: &BaseStatus) -> Report {
    report.fields.extend([
        ("base".to_string(), OutValue::Text(status.base.to_string())),
        ("partners".to_string(), OutValue::Raw(partners_json(&status.partners))),
        ("partner_moved".to_string(), OutValue::Bool(!status.moved.is_empty())),
    ]);
    if status.partners.is_empty() {
        return report;
    }
    report.lines.push(format!("base {}", status.base));
    report.lines.push(partners_line(&status.partners));
    if !status.moved.is_empty() {
        report.lines.push(format!("partner zones moved since the base: {}", status.moved.join(", ")));
    }
    report
}
