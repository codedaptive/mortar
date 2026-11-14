//! Attempt folders (S-69, S-82): making a request's attempts, keeping one and dropping them all, and as data the
//! attempts to make, the attempts a repository holds, the attempt folders whose branch is gone, what of a request's
//! attempts is left, which are unsaved, what keeping one removes, deletes and moves, and what dropping them removes
//! and deletes.

mod attempt;
mod attempt_refs_left;
mod attempts_of;
mod attempts_remaining;
mod drop_attempts;
mod drop_op;
mod drop_plan;
mod dropped;
mod keep;
mod keep_op;
mod keep_plan;
mod kept;
mod make_attempts;
mod orphan_attempts;
mod plan_attempts;
mod planned_attempt;
mod request_held;
mod unsaved_attempts;

#[cfg(test)]
mod attempt_refs_left_tests;
#[cfg(test)]
mod attempts_of_tests;
#[cfg(test)]
mod drop_op_tests;
#[cfg(test)]
mod keep_op_tests;
#[cfg(test)]
mod orphan_attempts_tests;
#[cfg(test)]
mod plan_attempts_tests;
#[cfg(test)]
mod request_held_tests;
#[cfg(test)]
mod unsaved_attempts_tests;

pub use attempt::Attempt;
pub use attempts_of::attempts_of;
pub use attempts_remaining::attempts_remaining;
pub use drop_attempts::drop_attempts;
pub use drop_op::drop_op;
pub use drop_plan::drop_plan;
pub use dropped::Dropped;
pub use keep::keep;
pub use keep_op::keep_op;
pub use keep_plan::keep_plan;
pub use kept::Kept;
pub use make_attempts::make_attempts;
pub use orphan_attempts::orphan_attempts;
pub use plan_attempts::plan_attempts;
pub use planned_attempt::PlannedAttempt;
pub use request_held::request_held;
pub(crate) use unsaved_attempts::unsaved_attempts;
