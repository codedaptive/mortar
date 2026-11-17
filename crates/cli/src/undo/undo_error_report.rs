//! undo_error_report: the report of an undo the op log refused or failed, one arm per OplogError, in the cli's own
//! words: no arm prints oplog's Display or an inner git or io error's (S-128 (4)).

use oplog::OplogError;

use crate::report::{failure_report, ExitCode, Report};

const VERB: &str = "undo";
/// The snapshot is taken before oplog's undo, so a refusal keeps it.
const KEPT: &str = "nothing was undone; the snapshot taken first is kept";

/// Refused (exit 1): MovedSince, naming the op, the ref and the op and actor that moved it (or that it moved outside
/// mortar), to the zone's coordinator; Protected, to its protector; Mixed, an open op with its refs half moved,
/// whose next asks the zone's coordinator in words to settle the op (a repeat meets the same refs, S-112). Every
/// other arm is failure_report, naming what failed.
pub fn undo_error_report(error: &OplogError) -> Report {
    match error {
        OplogError::MovedSince { op_id, refname, moved_by, .. } => {
            let (by, owner) = match moved_by {
                Some((op, actor)) => (format!("by op {op} of {actor}"), Some(actor.clone())),
                None => ("outside mortar".to_string(), None),
            };
            refused(
                format!("op {op_id} cannot be undone: {refname} moved since, {by}"),
                owner,
                format!("ask the zone's coordinator about {refname}"),
            )
        }
        OplogError::Protected { refname, by, .. } => refused(
            format!("{refname} is protected by {by}, and the undo is not a fast-forward of it"),
            Some(by.clone()),
            format!("ask {by} about {refname}"),
        ),
        OplogError::Mixed { op_id, refs, .. } => {
            // An agent runs only the six verbs, so the next is a request in words, not a harness command.
            let on = refs.first().map_or(String::new(), |(one, _)| format!(" on {}", one.refname));
            refused(
                format!("op {op_id} is open with its refs neither all moved nor all unmoved"),
                None,
                format!("ask the zone's coordinator to settle op {op_id}{on}"),
            )
        }
        OplogError::Malformed { line: Some(line), .. } => {
            failure_report(VERB, &format!("line {line} of the op log is not a record mortar wrote"))
        }
        OplogError::Malformed { line: None, .. } => failure_report(VERB, "the op log holds data mortar did not write"),
        OplogError::Io { path, .. } => failure_report(
            VERB,
            &format!("the op log at {} could not be read or written", path.display()),
        ),
        OplogError::Git(_) => failure_report(VERB, "git failed while undoing"),
        OplogError::BadMove { refname, .. } => failure_report(
            VERB,
            &format!("the undo would move {refname}, which is not a ref name git accepts"),
        ),
        OplogError::NothingToUndo { refs } => failure_report(
            VERB,
            &format!("the op log holds no done op that moved {}", refs.join(", ")),
        ),
        OplogError::UnknownOp { op_id } => failure_report(VERB, &format!("op {op_id} is not in the op log")),
        OplogError::ValueUnknown { op_id, refname } => failure_report(
            VERB,
            &format!("the value of {refname} as of op {op_id} is unknown"),
        ),
        OplogError::NotOpen { op_id } => {
            failure_report(VERB, &format!("op {op_id} has no open intent in the op log"))
        }
    }
}

fn refused(why: String, owner: Option<String>, next: String) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![why, KEPT.to_string()],
        owner,
        next,
        fields: Vec::new(),
    }
}
