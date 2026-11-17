//! Attempt folders (S-69, S-82): making a request's attempts, re-making its orphan attempts from their snapshots
//! (S-98), keeping one and dropping them all, and as data the attempts to make, the attempts a repository holds, the
//! attempt folders whose branch is gone, what of a request's attempts is left, which are unsaved, what keeping one
//! removes, deletes and moves, what dropping them removes and deletes, and how an orphan attempt is re-made from its
//! snapshot: at which tree and parent, with which message and trailers.

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
mod remade_message;
mod remade_trailers;
mod remake;
mod remake_orphans;
mod remake_plan;
mod request_held;
mod snap;
mod unsaved_attempts;

#[cfg(test)]
mod attempt_refs_left_tests;
#[cfg(test)]
mod attempts_of_tests;
#[cfg(test)]
mod attempts_remaining_tests;
#[cfg(test)]
mod drop_op_tests;
#[cfg(test)]
mod drop_plan_tests;
#[cfg(test)]
mod keep_op_tests;
#[cfg(test)]
mod keep_plan_tests;
#[cfg(test)]
mod orphan_attempts_tests;
#[cfg(test)]
mod plan_attempts_tests;
#[cfg(test)]
mod remade_message_tests;
#[cfg(test)]
mod remade_trailers_tests;
#[cfg(test)]
mod remake_plan_tests;
#[cfg(test)]
mod request_held_tests;
#[cfg(test)]
mod unsaved_attempts_tests;

pub use attempt::Attempt;
pub use attempts_of::attempts_of;
pub(crate) use attempts_remaining::attempts_remaining;
pub use drop_attempts::drop_attempts;
pub(crate) use drop_op::drop_op;
pub(crate) use drop_plan::drop_plan;
pub(crate) use dropped::Dropped;
pub use keep::keep;
pub(crate) use keep_op::keep_op;
pub(crate) use keep_plan::keep_plan;
pub(crate) use kept::Kept;
pub use make_attempts::make_attempts;
pub(crate) use orphan_attempts::orphan_attempts;
pub(crate) use plan_attempts::plan_attempts;
pub(crate) use planned_attempt::PlannedAttempt;
pub(crate) use remade_message::remade_message;
pub(crate) use remade_trailers::remade_trailers;
pub(crate) use remake::Remake;
pub use remake_orphans::remake_orphans;
pub(crate) use remake_plan::remake_plan;
pub(crate) use request_held::request_held;
pub(crate) use snap::Snap;
pub(crate) use unsaved_attempts::unsaved_attempts;
