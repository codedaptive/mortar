//! The reason an undo op records, "undo of <kind> op <op_id>" (S-142 (3)), its reading back, and the reading of the
//! reversed op's record: its Done record and whether it was a resolve's final join step (S-142, S-143). A leaf that
//! depends on no module above it, so undo writes the reason and join reads it without a join <-> undo cycle.

mod done_record;
mod is_resolve_step;
mod reversed_op;
mod undo_reason;

#[cfg(test)]
mod done_record_tests;
#[cfg(test)]
mod is_resolve_step_tests;
#[cfg(test)]
mod reversed_op_tests;
#[cfg(test)]
mod undo_reason_tests;

pub(crate) use done_record::done_record;
pub(crate) use is_resolve_step::is_resolve_step;
pub(crate) use reversed_op::reversed_op;
pub(crate) use undo_reason::undo_reason;
