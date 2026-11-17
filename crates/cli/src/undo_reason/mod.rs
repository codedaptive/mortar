//! The reason an undo op records, "undo of <kind> op <op_id>" (S-142 (3)), and its reading back: a leaf that
//! depends on no module above it, so undo writes the reason and join reads it without a join <-> undo cycle.

mod reversed_op;
mod undo_reason;

#[cfg(test)]
mod reversed_op_tests;
#[cfg(test)]
mod undo_reason_tests;

pub use reversed_op::reversed_op;
pub use undo_reason::undo_reason;
