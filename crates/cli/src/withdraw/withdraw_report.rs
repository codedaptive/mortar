//! The report of an `mr withdraw` that land's withdraw answered.

use land::WithdrawAnswer;

use crate::report::{ExitCode, OutValue, Report};

/// - Withdrawn: exit 0 "withdrawn", naming the commit the squash ref held and its change ID ("no change ID" when it
///   carried none), and that only that ref was deleted; next is a new squash, which draws a new change ID. Fields
///   zone, withdrawn (the commit) and change_id (null when none).
/// - Absent: exit 0 "nothing-to-withdraw", the zone has no squash ref and nothing was written; next "nothing to do".
///   Fields zone, withdrawn null and change_id null.
pub fn withdraw_report(zone: &str, answer: &WithdrawAnswer) -> Report {
    let squash_ref = format!("refs/mortar/squash/{zone}");
    let (outcome, lines, next, withdrawn, change_id) = match answer {
        WithdrawAnswer::Withdrawn { commit, change_id } => {
            let change = change_id.as_ref().map_or("no change ID".to_string(), |id| format!("change {id}"));
            (
                "withdrawn",
                vec![
                    format!("withdrew zone {zone}'s squash: commit {commit} ({change})"),
                    format!("{squash_ref} is deleted; no other ref moved"),
                ],
                format!(
                    "mr squash {zone} --actor <name> makes a new squash, with a new change ID, when the zone is ready \
                     for review"
                ),
                OutValue::Text(commit.to_string()),
                change_id.as_ref().map_or(OutValue::Null, |id| OutValue::Text(id.to_string())),
            )
        }
        WithdrawAnswer::Absent => (
            "nothing-to-withdraw",
            vec![format!("zone {zone} has no squash at {squash_ref}; nothing was written")],
            "nothing to do".to_string(),
            OutValue::Null,
            OutValue::Null,
        ),
    };
    Report {
        verb: "withdraw".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next,
        fields: vec![
            ("zone".to_string(), OutValue::Text(zone.to_string())),
            ("withdrawn".to_string(), withdrawn),
            ("change_id".to_string(), change_id),
        ],
    }
}
