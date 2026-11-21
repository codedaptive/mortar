//! repeat_head_refusal: head_refusal with the next of every verb but save, an attempt folder's snapshot and status in
//! an attempt folder, which name their own. It is the one source
//! of that next for snapshot (save/snapshot_error_report.rs), join and resolve (through snapshot_step_failure),
//! crate::join::snapshot_failure's callers, need (need/read_need_path.rs) and status outside an attempt
//! folder (status/saved_state_report.rs; disk keeps only its lines).

use super::{head_refusal, Report};

/// head_refusal under `verb` with `lines`, next checking out the request's branch in the folder and repeating the
/// same command.
pub(crate) fn repeat_head_refusal(verb: &str, lines: Vec<String>) -> Report {
    head_refusal(verb, lines, "check out the request's branch in the folder, then repeat the same command")
}
