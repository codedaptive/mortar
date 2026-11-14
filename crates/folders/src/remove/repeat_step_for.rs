//! Where a branch stands, after its delete op was answered as a repeat, against the tip it is deleted from.

use gitio::Oid;

use super::RepeatStep;

/// `now` (the branch's commit, None when absent) absent is Removed; at `tip` is Again; anywhere else is Moved.
pub(crate) fn repeat_step_for(now: Option<&Oid>, tip: &Oid) -> RepeatStep {
    match now {
        None => RepeatStep::Removed,
        Some(now) if now == tip => RepeatStep::Again,
        Some(_) => RepeatStep::Moved,
    }
}
