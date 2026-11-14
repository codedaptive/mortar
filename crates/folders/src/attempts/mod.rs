//! Attempt folders (S-69): making a request's attempts and keeping one, and as data the attempts to make, the
//! attempts a repository holds, and what keeping one removes, deletes and moves.

mod attempt;
mod attempts_of;
mod keep;
mod keep_op;
mod keep_plan;
mod kept;
mod make_attempts;
mod plan_attempts;
mod planned_attempt;
mod request_held;

#[cfg(test)]
mod attempts_of_tests;
#[cfg(test)]
mod keep_op_tests;
#[cfg(test)]
mod keep_plan_tests;
#[cfg(test)]
mod plan_attempts_tests;
#[cfg(test)]
mod request_held_tests;

pub use attempt::Attempt;
pub use attempts_of::attempts_of;
pub use keep::keep;
pub use keep_op::keep_op;
pub use keep_plan::keep_plan;
pub use kept::Kept;
pub use make_attempts::make_attempts;
pub use plan_attempts::plan_attempts;
pub use planned_attempt::PlannedAttempt;
pub use request_held::request_held;
