//! Removing a request folder once its work has joined: its worktree, its branch and its snapshot ref.

mod attempts_refusal;
mod delete_folder_refs;
mod remove;
mod remove_op;
mod remove_step;
mod remove_step_for;
mod repeat_step;
mod repeat_step_for;
mod step_refusal;
mod unsaved_refusal;

#[cfg(test)]
mod remove_op_tests;
#[cfg(test)]
mod remove_step_for_tests;
#[cfg(test)]
mod remove_tests;
#[cfg(test)]
mod repeat_step_for_tests;
#[cfg(test)]
mod step_refusal_tests;
#[cfg(test)]
mod unsaved_refusal_tests;

pub use remove::remove;
pub use remove_op::remove_op;
pub(crate) use remove_step::RemoveStep;
pub(crate) use repeat_step::RepeatStep;
