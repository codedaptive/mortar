//! The lines, fields, exit, outcome and next step a join answer gives its report.
//! Shell API: join_report and resolve_report call it.

use ::join::JoinAnswer;

use super::AnswerPart;
use crate::report::{ExitCode, OutValue};

/// Joined and FastForwarded are exit 0 "joined", next "wait for the zone's tests"; Rebased is exit 0 "rebased", next
/// "mr join", since the unit tests must run again first; Conflicted is exit 2 "conflicted", each file
/// indented, then a line saying mr resolve writes both sides into those files with markers to fix before running mr
/// resolve again, next "mr resolve". FilesWritten, which only resolve gives, is None. `moved` (moved_before's line)
/// names the refs the save or snapshot before the join moved: Joined says it before "nothing else moved",
/// Conflicted before "no other ref moved", and FastForwarded and Rebased at the end of their first line; None keeps
/// "nothing moved", "no ref moved" and FastForwarded's and Rebased's lines as they are.
pub fn answer_part(answer: &JoinAnswer, request_ref: &str, moved: Option<&str>) -> Option<AnswerPart> {
    const JOINED_NEXT: &str = "wait for the zone's tests";
    Some(match answer {
        JoinAnswer::Joined { zone_ref, request_tip } => AnswerPart {
            lines: vec![match moved {
                None => format!("joined: the request tip {request_tip} is already in {zone_ref}; nothing moved"),
                Some(moved) => format!(
                    "joined: the request tip {request_tip} is already in {zone_ref}; {moved}; nothing else moved"
                ),
            }],
            fields: vec![text("zone_ref", zone_ref), text("request_tip", request_tip)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: JOINED_NEXT,
        },
        JoinAnswer::FastForwarded { zone_ref, from, to } => AnswerPart {
            lines: vec![format!("joined: {zone_ref} moved forward from {from} to {to}{}", then_moved(moved))],
            fields: vec![text("zone_ref", zone_ref), text("from", from), text("to", to)],
            exit: ExitCode::Done,
            outcome: "joined",
            next: JOINED_NEXT,
        },
        JoinAnswer::Rebased { from, to, .. } => AnswerPart {
            lines: vec![
                format!(
                    "rebased: {request_ref} moved from {from} to {to}, replayed onto the zone; the folder moved with \
                     it{}",
                    then_moved(moved)
                ),
                "the unit tests must run again".to_string(),
            ],
            fields: vec![text("from", from), text("to", to)],
            exit: ExitCode::Done,
            outcome: "rebased",
            next: "mr join",
        },
        JoinAnswer::Conflicted { commit, files, state } => {
            let paths: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let refs = moved.map_or("no ref moved".to_string(), |moved| format!("{moved}; no other ref moved"));
            let mut lines = vec![format!(
                "conflicted: commit {commit} did not apply; {refs} and the folder stays at the request tip"
            )];
            lines.extend(paths.iter().map(|path| format!("  {path}")));
            lines.push(
                "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again"
                    .to_string(),
            );
            AnswerPart {
                lines,
                fields: vec![
                    text("commit", commit),
                    ("files".to_string(), OutValue::Texts(paths)),
                    text("state", state.display()),
                ],
                exit: ExitCode::Conflict,
                outcome: "conflicted",
                next: "mr resolve",
            }
        }
        JoinAnswer::FilesWritten { .. } => return None,
    })
}

/// "; <moved>" to end a line that names what moved, or nothing.
fn then_moved(moved: Option<&str>) -> String {
    moved.map_or(String::new(), |moved| format!("; {moved}"))
}

fn text(name: &str, value: impl ToString) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}
