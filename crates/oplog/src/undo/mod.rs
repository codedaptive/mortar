//! Undo and restore: reversing the last done op on some refs, or setting refs to their values as of an op, each as one new op through apply_protected.

mod caller_op;
mod find_undo_target;
mod last_mover;
mod name_moves;
mod op_caller;
mod plan_restore;
mod plan_undo;
mod restore;
mod undo;
mod value_after_move;
mod value_as_of_op;
mod zero_oid_like;

#[cfg(test)]
mod caller_op_tests;
#[cfg(test)]
mod find_undo_target_tests;
#[cfg(test)]
mod last_mover_tests;
#[cfg(test)]
mod name_moves_tests;
#[cfg(test)]
mod plan_restore_tests;
#[cfg(test)]
mod plan_undo_tests;
#[cfg(test)]
mod value_after_move_tests;
#[cfg(test)]
mod value_as_of_op_tests;
#[cfg(test)]
mod zero_oid_like_tests;

pub use caller_op::caller_op;
pub use find_undo_target::find_undo_target;
pub use last_mover::last_mover;
pub use op_caller::OpCaller;
pub use plan_restore::plan_restore;
pub use plan_undo::plan_undo;
pub use restore::restore;
pub use undo::undo;
pub use value_as_of_op::value_as_of_op;

pub(crate) use name_moves::name_moves;
pub(crate) use value_after_move::value_after_move;
pub(crate) use zero_oid_like::zero_oid_like;
