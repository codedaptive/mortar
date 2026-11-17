//! Protected refs: a protected ref moves only by fast-forward or creation (S-29); the pure verdict, the git reads, and apply behind both.

mod apply_protected;
mod check_protected_moves;
mod protected_move_verdict;
mod protected_ref;

#[cfg(test)]
mod protected_move_verdict_tests;

pub use apply_protected::apply_protected;
pub use protected_ref::ProtectedRef;

pub(crate) use check_protected_moves::check_protected_moves;
pub(crate) use protected_move_verdict::protected_move_verdict;
