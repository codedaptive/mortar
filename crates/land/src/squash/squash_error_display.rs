//! Display for SquashError: each variant says why and names what it carries;
//! the refusals squash makes itself also say what moved, and read_squash's say what to do.

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
            SquashError::TrailerMissing { commit, trailer } => write!(
                f,
                "commit {commit} has no {trailer} trailer, so it is not a squash mortar made; name the commit at \
                 refs/mortar/squash/<zone>, or squash the zone again"
            ),
            SquashError::TrailerMalformed { commit, trailer, value } => write!(
                f,
                "commit {commit} has a malformed {trailer} trailer {value:?}, so it cannot be read as a squash; \
                 squash the zone again to write a new squash commit"
            ),
            SquashError::RequestsMalformed { commit } => write!(
                f,
                "commit {commit}'s message does not list its requests as squash reads them (one `- <request>: \
                 ...` line each in the paragraph before the trailers, as many as a `<zone>: <n> requests` subject \
                 counts when the subject is squash's own); squash the zone again to write a new squash commit"
            ),
        }
    }
}
