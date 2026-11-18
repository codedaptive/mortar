//! install_report: Install's report once its plan is applied (spec 07 item 8).
//! Pure: it renders the plan as decided; it runs only after every step applied.

use crate::report::{ExitCode, OutValue, Report};

use super::{item_json, item_line, InstallPlan};

/// Exit 0, "installed" when a step ran else "already-installed", a line per item, the plan's next, and the items
/// as the --json member "items".
pub fn install_report(plan: &InstallPlan) -> Report {
    let outcome = if plan.steps.is_empty() { "already-installed" } else { "installed" };
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: plan.items.iter().map(item_line).collect(),
        owner: None,
        next: plan.next.clone(),
        fields: vec![("items".to_string(), OutValue::Raw(plan.items.iter().map(item_json).collect()))],
    }
}
