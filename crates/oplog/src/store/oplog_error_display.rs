//! How an op log refusal reads: each names its line, path, op or refs, and says what to do.

use std::fmt;

use gitio::Oid;

use super::OplogError;

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
                    " nothing was changed; find who moved them with history by ref, then close the op by hand \
                     with a done or failed record in the op log, since recover refuses every command until it \
                     is closed",
                )
            }
            OplogError::BadMove { refname, reason } => write!(
                f,
                "op cannot be applied: ref {refname:?} {reason}; fix the op's moves and run again"
            ),
        }
    }
}

/// The hex, or "absent" for a ref that does not exist.
fn oid_text(oid: Option<&Oid>) -> &str {
    oid.map_or("absent", Oid::as_str)
}
