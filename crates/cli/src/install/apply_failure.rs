//! apply_failure: The report when an install step failed and nothing went unrecorded: whatever was made or placed
//! before it is in the record (spec 07 item 7).
//! Pure: builds the report from its arguments; apply_failure_tests pins its text.

use crate::report::{ExitCode, Report};

use super::RecordItem;

/// Exit Failure, outcome "failed": `error`, then each of `items` (what was made and placed, all in the record) as
/// one that `<name> uninstall` can remove, so the next step is a repeat to finish or `<name> uninstall`.
/// `command` is the plan's planned name. `<name>` is "mr" only when `command` is "mr" and a `RecordItem::Link` is
/// among `items` (install's one link step is the mr link, so a Link item means it was applied); otherwise
/// "mortar". The planned "mr" holds only once the link is there; mortar is the binary install placed or found, the
/// way in when the link was skipped or not made. A reinstall whose link was already in place records no Link step,
/// so it names mortar.
/// With no items, nothing was applied or made, so the next step is the same command again.
pub fn apply_failure(error: &str, items: &[RecordItem], command: &str) -> Report {
    let (lines, next) = if items.is_empty() {
        (vec!["no step was applied and no folder was made".to_string()], "repeat the same command".to_string())
    } else {
        let linked = items.iter().any(|item| matches!(item, RecordItem::Link { .. }));
        let name = if command == "mr" && linked { "mr" } else { "mortar" };
        (
            items
                .iter()
                .map(|item| {
                    let (RecordItem::File { path, .. } | RecordItem::Link { path, .. } | RecordItem::Dir { path }) =
                        item;
                    format!("recorded, so {name} uninstall can remove it: {}", path.display())
                })
                .collect(),
            format!(
                "once the failure above is fixed, repeat the same command to finish the install, or run {name} \
                 uninstall to remove what was placed"
            ),
        )
    };
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: [error.to_string()].into_iter().chain(lines).collect(),
        owner: None,
        next,
        fields: Vec::new(),
    }
}
