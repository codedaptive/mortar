//! The report of an `mr would-merge` that land's would_merge answered (SPEC-cli 06).

use land::{Squash, WouldMerge};

use crate::landing::CHANGE_CONFLICT_NEXT;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "would-merge";
const NOTHING_MOVED: &str = "no ref moved and no commit was written";

/// - Yes: exit 0 "merges"; next lands the squash's commit. Fields change_id and commit.
/// - No: exit 2 "conflict", each conflicting file on its own line, owner the change's zone; next is
///   CHANGE_CONFLICT_NEXT. Fields change_id, commit and files.
///
/// Both say no ref moved and no commit was written (git's trial merge writes tree objects nothing keeps).
pub fn would_merge_report(squash: &Squash, answer: &WouldMerge) -> Report {
    let (id, commit) = (&squash.change_id, &squash.commit);
    let mut fields = vec![
        ("change_id".to_string(), OutValue::Text(id.to_string())),
        ("commit".to_string(), OutValue::Text(commit.to_string())),
    ];
    match answer {
        WouldMerge::Yes => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "merges".to_string(),
            lines: vec![
                format!("change {id} (commit {commit}) merges cleanly into integration"),
                NOTHING_MOVED.to_string(),
            ],
            owner: None,
            next: format!("mr land {commit} --actor <name>"),
            fields,
        },
        WouldMerge::No { files } => {
            let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let mut lines = vec![format!("change {id} (commit {commit}) conflicts with integration in:")];
            lines.extend(files.iter().cloned());
            lines.push(NOTHING_MOVED.to_string());
            fields.push(("files".to_string(), OutValue::Texts(files)));
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Conflict,
                outcome: "conflict".to_string(),
                lines,
                owner: Some("the change's zone".to_string()),
                next: CHANGE_CONFLICT_NEXT.to_string(),
                fields,
            }
        }
    }
}
