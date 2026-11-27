//! squash_pick_report: the refusal of a landing verb whose change ID names no squash ref, or more than one.

use oplog::ChangeId;

use super::{landing_refusal, SquashPick};
use crate::report::Report;

const OWNER: &str = "the zones' squash refs";

/// - One: None; the verb goes on with that ref's squash.
/// - Missing: refused, no squash ref carries the change ID; the next step makes the change's squash.
/// - Several: refused, naming each ref carrying it; the next step names the squash by its commit to `verb`.
///
/// Every refusal is exit 3, outcome "refused", with no fields.
pub fn squash_pick_report(verb: &str, change: &ChangeId, pick: &SquashPick) -> Option<Report> {
    match pick {
        SquashPick::One { .. } => None,
        SquashPick::Missing => Some(landing_refusal(
            verb,
            vec![format!("no squash at refs/mortar/squash/* carries change ID {change}")],
            OWNER,
            "make the change's squash with mr squash <zone> --actor <name>, then repeat the same command"
                .to_string(),
        )),
        SquashPick::Several { refnames } => Some(landing_refusal(
            verb,
            vec![format!("change ID {change} is carried by more than one squash ref: {}", refnames.join(", "))],
            OWNER,
            format!("name the squash by its commit instead: mr {verb} <commit> --actor <name>"),
        )),
    }
}
