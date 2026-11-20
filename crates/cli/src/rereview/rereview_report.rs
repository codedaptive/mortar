//! The report of an `mr rereview` that land answered: its re-review diff and whether the patch is the same.

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0 Done, no owner: one line per line of `answer.text`, in order ("the range-diff is empty" when it has
/// none). Fields: reviewed and new, each the commit id, then same_patch, `answer.same_patch`.
///
/// A different patch is outcome "rereviewed", next handing the diff to the reviewer of `reviewed`. The same patch
/// is outcome "same-patch", next saying `new` carries the same patch as `reviewed` and leaving to the caller
/// whether the review of `reviewed` still holds on the new base: the same patch says nothing about the base it
/// now sits on, so the report never says the review holds.
pub fn rereview_report(reviewed: &Oid, new: &Oid, answer: &land::RereviewAnswer) -> Report {
    let mut lines: Vec<String> = answer.text.lines().map(str::to_string).collect();
    if lines.is_empty() {
        lines.push("the range-diff is empty".to_string());
    }
    let (reviewed, new) = (reviewed.as_str(), new.as_str());
    let (outcome, next) = if answer.same_patch {
        (
            "same-patch",
            format!(
                "{new} carries the same patch as {reviewed}; whether the review of {reviewed} still holds on the new \
                 base is the caller's to decide"
            ),
        )
    } else {
        ("rereviewed", format!("hand this diff to the reviewer of {reviewed}"))
    };
    Report {
        verb: "rereview".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next,
        fields: vec![
            ("reviewed".to_string(), OutValue::Text(reviewed.to_string())),
            ("new".to_string(), OutValue::Text(new.to_string())),
            ("same_patch".to_string(), OutValue::Bool(answer.same_patch)),
        ],
    }
}
