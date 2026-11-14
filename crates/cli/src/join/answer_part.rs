//! The lines, fields, exit, outcome and next step a join answer gives its report (S-72, S-86).
//! Shell API: join_report and resolve_report call it.

use ::join::JoinAnswer;

use super::AnswerPart;
use crate::report::{ExitCode, OutValue};

/// Joined and FastForwarded are exit 0 "joined", next "wait for the zone's tests"; Rebased is exit 0 "rebased", next
/// "mortar join", since the unit tests must run again first (S-86); Conflicted is exit 2 "conflicted", each file
/// indented, next "mortar resolve". FilesWritten, which only resolve gives, is None.
pub fn answer_part(answer: &JoinAnswer, request_ref: &str) -> Option<AnswerPart> {
    const JOINED_NEXT: &str = "wait for the zone's tests";
    Some(match answer {
        JoinAnswer::Joined { zone_ref, request_tip } => AnswerPart {
            lines: vec![format!("joined: the request tip {request_tip} is already in {zone_ref}; nothing moved")],
            fields: vec![text("zone_ref", zone_ref), text("request_tip", request_tip)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: JOINED_NEXT,
        },
        JoinAnswer::FastForwarded { zone_ref, from, to } => AnswerPart {
            lines: vec![format!("joined: {zone_ref} moved forward from {from} to {to}")],
            fields: vec![text("zone_ref", zone_ref), text("from", from), text("to", to)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: JOINED_NEXT,
        },
        JoinAnswer::Rebased { from, to, .. } => AnswerPart {
            lines: vec![
                format!(
                    "rebased: {request_ref} moved from {from} to {to}, replayed onto the zone; the folder moved with it"
                ),
                "the unit tests must run again".to_string(),
            ],
            fields: vec![text("from", from), text("to", to)],
            exit: ExitCode::Done,
            outcome: "rebased",
            next: "mortar join",
        },
        JoinAnswer::Conflicted { commit, files, state } => {
            let paths: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let mut lines = vec![format!(
                "conflicted: commit {commit} did not apply; no ref moved and the folder stays at the request tip"
            )];
            lines.extend(paths.iter().map(|path| format!("  {path}")));
            AnswerPart {
                lines,
                fields: vec![
                    text("commit", commit),
                    ("files".to_string(), OutValue::Texts(paths)),
                    text("state", state.display()),
                ],
                exit: ExitCode::Conflict,
                outcome: "conflicted",
                next: "mortar resolve",
            }
        }
        JoinAnswer::FilesWritten { .. } => return None,
    })
}

fn text(name: &str, value: impl ToString) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}
