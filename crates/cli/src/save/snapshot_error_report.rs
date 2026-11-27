//! snapshot_error_report: the report of an `mr snapshot` that did not happen, one arm per SnapshotError, in the
//! cli's own words: no arm prints an inner git, op log, trailer, change ID or io error's Display.

use oplog::OpResult;
use snapshot::SnapshotError;

use crate::report::{failure_report, repeat_head_refusal, ExitCode, Report};

const VERB: &str = "snapshot";

/// Refused (exit 3): an unborn HEAD and a refused path that is not UTF-8 (both reached through snapshot_formatted),
/// and a detached HEAD (save's arm: only save_head refuses one; refused all the same). Busy (exit 4): the snapshot
/// ref moved, so nothing moved. Done (exit 0): the op was already done. Failure (exit 5): every other arm, naming
/// what happened; BadSentence, ChangeIdMissing, Trailers, ChangeId, BranchMoved and IndexNotSet are save's arms,
/// which snapshot_formatted does not return, worded all the same.
pub fn snapshot_error_report(error: &SnapshotError) -> Report {
    match error {
        SnapshotError::UnbornHead { .. } | SnapshotError::DetachedHead { .. } => {
            repeat_head_refusal(VERB, vec![error.to_string(), "nothing was snapshotted".to_string()])
        }
        SnapshotError::PathNotUtf8 { path } => refused(
            vec![format!(
                "the refused file {path:?} has a name that is not UTF-8, so it cannot be passed to git; nothing was \
                 snapshotted"
            )],
            format!("the file {}", path.display()),
            "rename the file to a UTF-8 name, then repeat the same command",
        ),
        SnapshotError::RefMoved(record) => {
            let line = match &record.result {
                Some(OpResult::Stale { refname, .. }) => {
                    format!("the snapshot ref {refname} moved while snapshot ran; nothing moved")
                }
                None | Some(OpResult::Moved) | Some(OpResult::NotMoved { .. }) => {
                    "the snapshot ref moved while snapshot ran; nothing moved".to_string()
                }
            };
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Busy,
                outcome: "busy".to_string(),
                lines: vec![line],
                owner: None,
                next: "repeat the same command later".to_string(),
                fields: Vec::new(),
            }
        }
        SnapshotError::OpRepeated(_) => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "already-done".to_string(),
            lines: vec!["this snapshot was already done".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: Vec::new(),
        },
        SnapshotError::BadTarget { reason } => {
            failure_report(VERB, &format!("the snapshot ref cannot be named: {reason}"))
        }
        SnapshotError::Git(_) => failure_report(VERB, "git failed while snapshotting the folder"),
        SnapshotError::Oplog(_) => failure_report(VERB, "the op log could not be read or written"),
        SnapshotError::FileSize { path, .. } => {
            failure_report(VERB, &format!("the size of {} could not be read", path.display()))
        }
        SnapshotError::CapMismatch { paths } => failure_report(VERB, &format!(
            "the capped tree did not differ from the full tree by the refused files only: {}; GIT_NOGLOB_PATHSPECS or \
             GIT_LITERAL_PATHSPECS may be set, or the folder changed while snapshot ran",
            paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
        )),
        SnapshotError::IndexNotSet { commit, .. } => {
            failure_report(VERB, &format!("commit {commit} was written but the folder's index was not set"))
        }
        SnapshotError::Trailers(_) => failure_report(VERB, "HEAD's Mortar trailers could not be read"),
        SnapshotError::ChangeId(_) => failure_report(VERB, "a new change ID could not be drawn"),
        SnapshotError::BadSentence { reason } => failure_report(VERB, &format!("the sentence was refused: {reason}")),
        SnapshotError::ChangeIdMissing { head } => {
            failure_report(VERB, &format!("HEAD {head} carries no Mortar-Change-Id"))
        }
        SnapshotError::BranchMoved(_) => failure_report(VERB, "the folder's branch moved while snapshot ran"),
    }
}

fn refused(lines: Vec<String>, owner: String, next: &str) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(owner),
        next: next.to_string(),
        fields: Vec::new(),
    }
}
