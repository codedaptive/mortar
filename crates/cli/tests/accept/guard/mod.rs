//! Step 2, the guard (02-guard.md): run_guard, its set-up and check groups in spec order, and the pure readings of
//! what git said when the guard refused it.

mod guard_commit;
mod guard_land;
mod guard_pushes;
mod guard_refused;
mod guard_scratch;
mod guard_squash;
mod local_ref;
mod other_commit;
mod refusal_ok;
mod refusal_parts;
mod run_guard;
mod shown_ref;

#[cfg(test)]
mod refusal_ok_tests;
#[cfg(test)]
mod refusal_parts_tests;
#[cfg(test)]
mod shown_ref_tests;

pub use guard_commit::guard_commit;
pub use guard_land::guard_land;
pub use guard_pushes::guard_pushes;
pub use guard_refused::guard_refused;
pub use guard_scratch::guard_scratch;
pub use guard_squash::guard_squash;
pub use local_ref::local_ref;
pub use other_commit::other_commit;
pub use refusal_ok::refusal_ok;
pub use refusal_parts::refusal_parts;
pub use run_guard::run_guard;
pub use shown_ref::shown_ref;
