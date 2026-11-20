//! Moving a folder between two tips: its files and index by gitio, its branch by one op.

mod branch_move;
mod branch_move_for;
mod move_attempt;
mod move_attempts;
mod move_folder;
mod move_op;
mod moved_or_refused;

#[cfg(test)]
mod branch_move_for_tests;
#[cfg(test)]
mod move_attempts_tests;
#[cfg(test)]
mod move_folder_tests;
#[cfg(test)]
mod move_op_tests;
#[cfg(test)]
mod moved_or_refused_tests;

pub(crate) use branch_move::BranchMove;
pub(crate) use move_attempt::MoveAttempt;
pub use move_folder::move_folder;
pub(crate) use move_op::move_op;
