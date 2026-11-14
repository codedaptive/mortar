//! Display for SquashError: each variant says why and names what it carries;
//! the refusals squash makes itself also say what moved.

use crate::squash::SquashError;
use std::fmt;

impl fmt::Display for SquashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquashError::Git(e) => write!(f, "{e}"),
            SquashError::Oplog(e) => write!(f, "squash op log: {e}"),
            SquashError::Trailers(e) => write!(f, "squash trailers: {e}"),
            SquashError::ChangeId(e) => write!(f, "squash change ID: {e}"),
            SquashError::RefMissing { refname } => {
                write!(f, "ref {refname} does not exist; no commit was written and no ref was moved")
            }
            SquashError::AttrSourceMissing { commit } => write!(
                f,
                "attribute source commit {commit} does not exist, so the merge cannot read its attributes; no \
                 commit was written and no ref was moved"
            ),
            SquashError::Unrelated { origin, source } => write!(
                f,
                "zone tip {source} shares no history with origin {origin}; nothing was squashed and no ref was \
                 moved"
            ),
            SquashError::OpFailed(record) => write!(
                f,
                "op {} (key {}) failed: the squash ref was not at its expected value, so nothing was moved; run \
                 squash again to read the refs afresh",
                record.op_id, record.key
            ),
            SquashError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this squash moved nothing; run squash again",
                record.key
            ),
        }
    }
}
