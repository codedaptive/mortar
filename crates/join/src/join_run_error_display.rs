//! Display for JoinRunError: each refusal says why and names the ref, path or op it carries.
//! join_run_error_display_tests and join_run_error_display_restore_tests cover each variant's text.

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
                "op {} (key {}) failed: a ref was not at its expected value, so nothing was moved; run the \
                 command again to read the refs afresh",
                record.op_id, record.key
            ),
            JoinRunError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this command moved nothing; run it again",
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
                 permissions, then run the command again",
                path.display()
            ),
            JoinRunError::NothingToResolve { state } => {
                write!(f, "nothing to resolve: {} does not exist, so no join is paused in this folder", state.display())
            }
            JoinRunError::StillMarked(files) => {
                write!(f, "conflict markers remain; no commit was made and no ref was moved:")?;
                for file in files {
                    let lines: Vec<String> = file.lines.iter().map(ToString::to_string).collect();
                    write!(f, "\n  {} (lines {})", file.path.display(), lines.join(", "))?;
                }
                write!(f, "\nremove them, then run `mr resolve` again")
            }
            JoinRunError::StateStale { request_ref, expected, actual, state } => {
                match actual {
                    Some(actual) => write!(f, "request ref {request_ref} is at {actual}, ")?,
                    None => write!(f, "request ref {request_ref} does not exist, ")?,
                }
                write!(
                    f,
                    "not at {expected} where the paused join left it, so {} no longer describes the request; the \
                     state file was kept and no ref was moved. To start over: remove {}, then run `mr join` to \
                     replay the request afresh",
                    state.display(),
                    state.display()
                )
            }
            JoinRunError::StateRead { path, source } => {
                write!(f, "{}: {source}; the join state could not be read and no ref was moved", path.display())
            }
            JoinRunError::StateCorrupt { path, reason } => write!(
                f,
                "{} is not a state join wrote: {reason}; resolve refuses and no ref was moved",
                path.display()
            ),
            JoinRunError::StateRemove { path, source } => write!(
                f,
                "{}: {source}; the request ref was moved but the join state was not removed; check that folder and \
                 its permissions, then run `mr resolve` again to finish",
                path.display()
            ),
            JoinRunError::FileRead { path, source } => write!(
                f,
                "{}: {source}; the conflicted file could not be checked for markers and no ref was moved",
                path.display()
            ),
            JoinRunError::ConflictedPathIsFolder { path } => write!(
                f,
                "conflicted path {} names a folder; nothing was committed and no ref was moved. A conflicted path must \
                 be a file or removed: make it one, then run `mr resolve` again",
                path.display()
            ),
            JoinRunError::ConflictedPathRefused { path, reason } => {
                match path {
                    Some(path) => write!(f, "conflicted path {} was refused: ", path.display())?,
                    None => write!(f, "the conflicted paths were refused: ")?,
                }
                write!(f, "{reason}; nothing was committed and no ref was moved")
            }
            JoinRunError::NotAResolveOp { op_id } => write!(
                f,
                "op {op_id} is not a resolve's final join-rebase op recording a paused join's state; nothing was \
                 written. Only a resolve's final op restores a paused join: undo that op as a plain step, or check \
                 the op id with `mr log`"
            ),
            JoinRunError::RecordedStateCorrupt { op_id, reason } => write!(
                f,
                "the state recorded in op {op_id} cannot be used: {reason}; nothing was written. The op's ref move \
                 can still be undone, but the paused join cannot be put back from it: run `mr join` to replay \
                 the request afresh"
            ),
            JoinRunError::RestoreStale { request_ref, expected, actual } => {
                match actual {
                    Some(actual) => write!(f, "request ref {request_ref} is at {actual}, ")?,
                    None => write!(f, "request ref {request_ref} does not exist, ")?,
                }
                write!(
                    f,
                    "not at {expected} where the recorded paused join left it; nothing was written: move \
                     {request_ref} back to {expected} (undo the op's ref move) first, then restore again"
                )
            }
            JoinRunError::StatePresent { state } => write!(
                f,
                "{} holds a different paused join; it was kept and nothing was written: finish that join with \
                 `mr resolve`, or remove {} by hand, then restore again",
                state.display(),
                state.display()
            ),
        }
    }
}
