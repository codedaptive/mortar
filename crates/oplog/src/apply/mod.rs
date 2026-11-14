//! Applying an op: the repeat-safe lookup, the ref name check, the intent, the one ref transaction, and its
//! closing record; and running work under a held lock, logging the lock's guard removals and takeover first.

mod apply;
mod apply_outcome;
mod check_ref_moves;
mod check_ref_names;
mod close_after_update;
mod find_done_record;
mod guard_removal_op;
mod guard_removal_ops;
mod held_lock_error;
mod new_op_id;
mod op_id_from_parts;
mod ref_format_verdict;
mod ref_updates;
mod takeover_op;
mod with_lock;

#[cfg(test)]
mod check_ref_moves_tests;
#[cfg(test)]
mod close_after_update_tests;
#[cfg(test)]
mod find_done_record_tests;
#[cfg(test)]
mod guard_removal_op_tests;
#[cfg(test)]
mod guard_removal_ops_tests;
#[cfg(test)]
mod op_id_from_parts_tests;
#[cfg(test)]
mod ref_format_verdict_tests;
#[cfg(test)]
mod ref_updates_tests;
#[cfg(test)]
mod takeover_op_tests;

pub use apply::apply;
pub use apply_outcome::ApplyOutcome;
pub use check_ref_moves::check_ref_moves;
pub use check_ref_names::check_ref_names;
pub use close_after_update::close_after_update;
pub use find_done_record::find_done_record;
pub use guard_removal_op::guard_removal_op;
pub use held_lock_error::HeldLockError;
pub use new_op_id::new_op_id;
pub use op_id_from_parts::op_id_from_parts;
pub use ref_format_verdict::ref_format_verdict;
pub use ref_updates::ref_updates;
pub use takeover_op::takeover_op;
pub use with_lock::with_lock;
