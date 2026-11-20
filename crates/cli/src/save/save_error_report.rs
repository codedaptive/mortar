//! save_error_report: the report of a save that did not happen, one arm per SnapshotError.
//! The cli writes its own text per arm, for its own exit, owner and next; the inner git, op log, trailer and
//! change ID errors are shown through their own Display. mr save reports its Git, Trailers, ChangeId and Oplog errors
//! (from the pending move's snapshot, the formatter's snapshot, snapshot::save_from_snapshot and snapshot::save_with_snapshot) through
//! save_step_failure, which passes every other error here; mr status uses this report for the errors of its
//! saved_state read.

use oplog::{OpRecord, OpResult};
use snapshot::SnapshotError;

use crate::report::{failure_report, ExitCode, Report};

/// The step after a refused sentence.
const NEXT_SENTENCE: &str = "mr save \"<one line saying what changed>\"";
/// The step after a HEAD save cannot use.
const NEXT_HEAD: &str = "check out the request's branch in this folder, then mr save again";

/// Refused (exit 1): a bad sentence, an unborn or detached HEAD, a HEAD of this request without a change ID
/// (save will not make a second ID for the request), a refused path that is not UTF-8 (this snapshot wrote no
/// commit). Busy (exit 3): the branch moved while save ran, in two lines: the branch that moved (named when the
/// failed record's result is Stale), then that this save did not move the branch and the snapshot ref may have
/// moved; or the snapshot ref moved (one line, ending "this snapshot moved nothing"; what earlier steps moved is
/// save_step_failure's line before it). Done (exit 0): the op was already done (a repeat of a finished command is 0).
/// Failure (exit 5): every other error, through failure_report; IndexNotSet gives three lines: the commit whose tree
/// the index was to be set to (the new commit, or HEAD on an unchanged save) with git's error, that the branch stands
/// at that commit and the snapshot ref may have moved, and that the index step left the files untouched and a repeat
/// sets the index, skip-worktree bits included when the sparse-checkout reapply failed. The printed lines are plain
/// words with no reference numbers, since users read them.
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
                "no save commit was written; the snapshot ref may have moved".to_string(),
            ],
            Some(format!("the commit {head}")),
            "ask the zone's coordinator to mend HEAD's trailers",
        ),
        SnapshotError::PathNotUtf8 { path } => refused(
            vec![format!(
                "the refused file {path:?} has a name that is not UTF-8, so it cannot be passed to git; this \
                 snapshot wrote no commit"
            )],
            Some(format!("the file {}", path.display())),
            "rename the file to a UTF-8 name, then mr save again",
        ),
        SnapshotError::BranchMoved(record) => busy(vec![
            moved_line("the branch", record),
            "this save did not move the branch; the snapshot ref may have moved".to_string(),
        ]),
        SnapshotError::RefMoved(record) => {
            busy(vec![format!("{}; this snapshot moved nothing", moved_line("the snapshot ref", record))])
        }
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
                format!("the folder's index could not be set to commit {commit}'s tree: {error}"),
                format!("the branch stands at commit {commit}; the snapshot ref may have moved"),
                "the index step left the files untouched; the index is the old one, or the saved tree without its \
                 skip-worktree bits if the sparse-checkout reapply failed; a repeat sets it"
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
    refused(vec![why, "no save commit was written".to_string()], Some("the folder's HEAD".to_string()), NEXT_HEAD)
}

fn busy(lines: Vec<String>) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Busy,
        outcome: "busy".to_string(),
        lines,
        owner: None,
        next: "repeat the same command later".to_string(),
        fields: Vec::new(),
    }
}

/// That `what` moved while save ran: the failed record's Stale result names the ref that moved; any other record
/// names none. The caller adds what this save left: a branch apply refuses after save's snapshot step, which may
/// already have moved the snapshot ref.
fn moved_line(what: &str, record: &OpRecord) -> String {
    match &record.result {
        Some(OpResult::Stale { refname, .. }) => format!("{what} {refname} moved while save ran"),
        _ => format!("{what} moved while save ran"),
    }
}
