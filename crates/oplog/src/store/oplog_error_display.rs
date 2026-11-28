//! How an op log refusal reads: each says what to do (a Protected delete does not) and names its
//! line, path, op or refs where it has one. A Protected refusal of a delete is told by an
//! all-zero new value (oid_is_zero).

use std::fmt;

use gitio::Oid;

use super::{oid_is_zero, OplogError};

/// A Protected refusal of a delete gives no next step on purpose: this text reaches a person only inside the
/// caller's report, which gives the next step (cli's join_refusal_report does).
impl fmt::Display for OplogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OplogError::Malformed {
                line: Some(line),
                reason,
            } => write!(
                f,
                "op log line {line} is not a record ({reason}); mortar did not write it, so check what did, \
                 then repair or remove that line of mortar/oplog.jsonl"
            ),
            OplogError::Malformed { line: None, reason } => write!(
                f,
                "malformed op data: {reason}; mortar did not write this text, so check what did"
            ),
            OplogError::Io { path, source } => write!(
                f,
                "{}: {source}; check that path's folder and its permissions, then run again",
                path.display()
            ),
            OplogError::Git(e) => write!(f, "{e}; fix what git reports, then run again"),
            OplogError::Mixed { op_id, key, refs } => {
                write!(
                    f,
                    "op {op_id} (key {key}) is open and its refs are neither all at their new values nor all \
                     at their old ones:"
                )?;
                for (planned, actual) in refs {
                    write!(
                        f,
                        " {} planned {} -> {}, now {};",
                        planned.refname,
                        oid_text(planned.old.as_ref()),
                        planned.new.as_str(),
                        oid_text(actual.as_ref())
                    )?;
                }
                f.write_str(
                    " nothing was changed; find who moved them with mr log --ref <ref>, then close the op \
                     by hand with a done or failed record in the op log, since recover refuses every command \
                     until it is closed",
                )
            }
            OplogError::BadMove { refname, reason } => write!(
                f,
                "op cannot be applied: ref {refname:?} {reason}; fix the op's moves and run again"
            ),
            OplogError::Protected {
                refname,
                current,
                new,
                by,
            } => {
                let cur = oid_text(current.as_ref());
                if oid_is_zero(new) {
                    write!(
                        f,
                        "ref {refname} is protected by {by}: deleting it is refused (it holds {cur}); nothing was \
                         changed"
                    )
                } else {
                    write!(
                        f,
                        "ref {refname} is protected by {by}: moving it from {cur} to {} is not a fast-forward; \
                         nothing was changed; move it only to a commit that has {cur} as an ancestor",
                        new.as_str()
                    )
                }
            }
            OplogError::NothingToUndo { refs } => write!(
                f,
                "nothing to undo: no done op in the op log moved {}; check the ref names with mr log \
                 --ref <ref>",
                refs.join(", ")
            ),
            OplogError::MovedSince {
                op_id,
                refname,
                left,
                actual,
                moved_by,
            } => {
                let left = oid_text(left.as_ref());
                let actual = oid_text(actual.as_ref());
                match moved_by {
                    Some((mop, actor)) => write!(
                        f,
                        "cannot undo op {op_id}: ref {refname} moved since (that op left it at {left}, it now \
                         holds {actual}), by op {mop} of {actor}; undo op {mop} first, or restore the ref to the \
                         value you want"
                    ),
                    None => write!(
                        f,
                        "cannot undo op {op_id}: ref {refname} moved since (that op left it at {left}, it now \
                         holds {actual}) outside mortar: no logged op explains it; find who moved it, then \
                         restore the ref to the value you want"
                    ),
                }
            }
            OplogError::UnknownOp { op_id } => write!(
                f,
                "cannot restore: op {op_id} is not in the op log; find the op id with mr log"
            ),
            OplogError::ValueUnknown { op_id, refname } => write!(
                f,
                "cannot restore ref {refname} as of op {op_id}: no done op in the op log moved it, so its value \
                 then is unknown; name only refs mortar has moved"
            ),
            OplogError::NotOpen { op_id } => write!(
                f,
                "cannot close op {op_id}: it has no open intent in the op log (never written, or already \
                 closed); nothing was written; in mortar/oplog.jsonl, find the intent line that no later \
                 done or failed line closes and use its op_id"
            ),
        }
    }
}

/// The hex, or "absent" for a ref that does not exist.
fn oid_text(oid: Option<&Oid>) -> &str {
    oid.map_or("absent", Oid::as_str)
}
