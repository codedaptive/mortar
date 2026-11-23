//! Protected refs: a protected ref moves only by fast-forward or creation; the pure verdict, the git reads, and
//! apply_protected and apply_protected_with_values behind both.

mod apply_protected;
mod apply_protected_with_values;
mod check_protected_moves;
mod protected_move_verdict;
mod protected_ref;

#[cfg(test)]
mod protected_move_verdict_tests;

pub use apply_protected::apply_protected;
pub use apply_protected_with_values::apply_protected_with_values;
pub use protected_ref::ProtectedRef;

pub(crate) use check_protected_moves::check_protected_moves;
pub(crate) use protected_move_verdict::protected_move_verdict;
