//! save_error_report: the report of a save that did not happen, one arm per SnapshotError (S-71).
//! The cli writes its own text per arm, for its own exit, owner and next (S-71); the inner git, op log, trailer and
//! change ID errors are shown through their own Display.

use oplog::{OpRecord, OpResult};
use snapshot::SnapshotError;

use crate::report::{failure_report, ExitCode, Report};

/// The step after a refused sentence.
const NEXT_SENTENCE: &str = "mr save \"<one line saying what changed>\"";
/// The step after a HEAD save cannot use.
const NEXT_HEAD: &str = "check out the request's branch in this folder, then mr save again";

/// Refused (exit 1): a bad sentence, an unborn or detached HEAD, a HEAD of this request without a change ID (S-34:
/// save will not make a second ID for the request), a refused path that is not UTF-8. Busy (exit 3): the branch
/// moved while save ran (the snapshot ref may have moved), or the snapshot ref moved (nothing moved). Done (exit 0):
/// the op was already done (S-71: a repeat of a finished command is 0). Failure (exit 5): every other error, through
/// failure_report; IndexNotSet says the files are untouched and a repeat sets the index (S-41), skip-worktree bits
/// included when the sparse-checkout reapply failed (S-125). The printed lines cite no ruling numbers: users read
/// them, and the rulings are for maintainers.
pub fn save_error_report(error: &SnapshotError) -> Report {
    match error {
        SnapshotError::BadSentence { reason } => {
            refused(vec![reason.clone(), "nothing ran".to_string()], None, NEXT_SENTENCE)
        }
        SnapshotError::UnbornHead { folder } => {
            head_refused(format!("HEAD of the folder {} has no commit", folder.display()))
        }
        SnapshotError::DetachedHead { folder } => {
            head_refused(format!("HEAD of the folder {} names no branch", folder.display()))
        }
        SnapshotError::ChangeIdMissing { head } => refused(
            vec![
                format!("HEAD {head} is this request's but carries no Mortar-Change-Id"),
                "no commit was written; the snapshot ref may have moved".to_string(),
            ],
            Some(format!("the commit {head}")),
            "ask the zone's coordinator to mend HEAD's trailers",
        ),
        SnapshotError::PathNotUtf8 { path } => refused(
            vec![format!(
                "the refused file {path:?} has a name that is not UTF-8, so it cannot be passed to git; no commit \
                 was written"
            )],
            Some(format!("the file {}", path.display())),
            "rename the file to a UTF-8 name, then mr save again",
        ),
        SnapshotError::BranchMoved(record) => busy(moved_line(
            "the branch",
            record,
            "this save did not move it, and the snapshot ref may have moved",
        )),
        SnapshotError::RefMoved(record) => busy(moved_line("the snapshot ref", record, "nothing moved")),
        SnapshotError::OpRepeated(_) => Report {
            verb: "save".to_string(),
            exit: ExitCode::Done,
            outcome: "already-done".to_string(),
            lines: vec!["this save was already done".to_string()],
            owner: None,
            next: "mr join".to_string(),
            fields: Vec::new(),
        },
        SnapshotError::IndexNotSet { commit, error } => Report {
            lines: vec![
                format!("saved commit {commit} but the folder's index was not set: {error}"),
                "the files are untouched and the index is the old one, or the saved tree without its skip-worktree \
                 bits if the sparse-checkout reapply failed; a repeat sets it"
                    .to_string(),
            ],
            ..failure_report("save", "")
        },
        SnapshotError::CapMismatch { paths } => failure_report("save", &format!(
            "the capped tree did not differ from the full tree by the refused files only: {}; GIT_NOGLOB_PATHSPECS or \
             GIT_LITERAL_PATHSPECS may be set, or the folder changed while save ran",
            paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
        )),
        SnapshotError::FileSize { path, error } => {
            failure_report("save", &format!("the size of {} could not be read: {error}", path.display()))
        }
        SnapshotError::BadTarget { reason } => {
            failure_report("save", &format!("the snapshot ref cannot be named: {reason}"))
        }
        SnapshotError::Git(error) => failure_report("save", &error.to_string()),
        SnapshotError::Oplog(error) => failure_report("save", &error.to_string()),
        SnapshotError::Trailers(error) => failure_report("save", &error.to_string()),
        SnapshotError::ChangeId(error) => failure_report("save", &error.to_string()),
    }
}

fn refused(lines: Vec<String>, owner: Option<String>, next: &str) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner,
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn head_refused(why: String) -> Report {
    refused(vec![why, "nothing was saved".to_string()], Some("the folder's HEAD".to_string()), NEXT_HEAD)
}

fn busy(line: String) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Busy,
        outcome: "busy".to_string(),
        lines: vec![line],
        owner: None,
        next: "repeat the same command later".to_string(),
        fields: Vec::new(),
    }
}

/// The failed record's Stale result names the ref that moved; any other record names none. `left` says what this
/// save left: a branch apply refuses after save's snapshot step, which may already have moved the snapshot ref.
fn moved_line(what: &str, record: &OpRecord, left: &str) -> String {
    match &record.result {
        Some(OpResult::Stale { refname, .. }) => format!("{what} {refname} moved while save ran; {left}"),
        _ => format!("{what} moved while save ran; {left}"),
    }
}
