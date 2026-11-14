//! Applying an op: the repeat-safe lookup, the ref name check, the intent, the one ref transaction run through
//! the op's git (MORTAR_OP naming the op for the guard hook), and its closing record; apply_with_values is the
//! entry point that writes the caller's values in the intent and closing record, apply the one with none; and
//! running work under a held lock, logging the lock's guard removals and takeover first; and opening a remote
//! intent, marked with the remote, with no ref transaction.

mod apply;
mod apply_outcome;
mod apply_with_values;
mod check_ref_moves;
mod check_ref_names;
mod close_after_update;
mod find_done_in;
mod find_done_record;
mod guard_removal_op;
mod guard_removal_ops;
mod held_lock_error;
mod mortar_op_env;
mod new_op_id;
mod op_git;
mod op_id_from_parts;
mod open_remote_intent;
mod ref_format_verdict;
mod ref_updates;
mod remote_intent_outcome;
mod remote_intent_verdict;
mod takeover_op;
mod valued_outcome;
mod with_lock;

#[cfg(test)]
mod check_ref_moves_tests;
#[cfg(test)]
mod close_after_update_tests;
#[cfg(test)]
mod find_done_in_tests;
#[cfg(test)]
mod find_done_record_tests;
#[cfg(test)]
mod guard_removal_op_tests;
#[cfg(test)]
mod guard_removal_ops_tests;
#[cfg(test)]
mod op_git_tests;
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
pub use apply_with_values::apply_with_values;
pub use check_ref_moves::check_ref_moves;
pub use check_ref_names::check_ref_names;
pub use close_after_update::close_after_update;
pub use find_done_record::find_done_record;
pub use guard_removal_op::guard_removal_op;
pub use held_lock_error::HeldLockError;
pub use mortar_op_env::MORTAR_OP_ENV;
pub use new_op_id::new_op_id;
pub use op_git::op_git;
pub use op_id_from_parts::op_id_from_parts;
pub use open_remote_intent::open_remote_intent;
pub use ref_format_verdict::ref_format_verdict;
pub use ref_updates::ref_updates;
pub use remote_intent_outcome::RemoteIntentOutcome;
pub use takeover_op::takeover_op;
pub use valued_outcome::ValuedOutcome;
pub use with_lock::with_lock;

pub(crate) use find_done_in::find_done_in;
pub(crate) use remote_intent_verdict::remote_intent_verdict;
