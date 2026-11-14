//! Display for JoinRunError: each refusal says why and names the ref, path or op it carries.

use crate::JoinRunError;
use std::fmt;

impl fmt::Display for JoinRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinRunError::Replay(e) => write!(f, "join replay: {e}; no ref was moved"),
            JoinRunError::Oplog(e) => write!(f, "join op log: {e}"),
            JoinRunError::Git(e) => write!(f, "{e}"),
            JoinRunError::ZoneMissing(zone_ref) => write!(
                f,
                "zone ref {zone_ref} does not exist; join does not create it: make the zone folder first"
            ),
            JoinRunError::RequestMissing(request_ref) => {
                write!(f, "request ref {request_ref} does not exist; nothing to join")
            }
            JoinRunError::Unrelated { zone_ref, request_ref } => write!(
                f,
                "request ref {request_ref} shares no history with zone ref {zone_ref}; join refuses, no ref was \
                 moved"
            ),
            JoinRunError::OpFailed(record) => write!(
                f,
                "op {} (key {}) failed: a ref was not at its expected value, so nothing was moved; run join again \
                 to read the refs afresh",
                record.op_id, record.key
            ),
            JoinRunError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this join moved nothing; run join again",
                record.key
            ),
            JoinRunError::StatePath(path) => write!(
                f,
                "conflicted path {} is not UTF-8, so mortar-join.json cannot record it; no ref was moved",
                path.display()
            ),
            JoinRunError::StateWrite { path, source } => write!(
                f,
                "{}: {source}; mortar-join.json was not written and no ref was moved; check that folder and its \
                 permissions, then run join again",
                path.display()
            ),
        }
    }
}
