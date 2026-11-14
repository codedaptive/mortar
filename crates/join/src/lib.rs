//! join: see the Mortar mission (Layout).

mod after_replay;
mod after_replay_step;
mod check_onto;
mod decide_join;
mod encode_join_state;
mod fast_forward_op;
mod git_replay_git;
mod join;
mod join_answer;
mod join_answer_display;
mod join_decision;
mod join_error;
mod join_error_display;
mod join_error_error;
mod join_error_from_git;
mod join_read;
mod join_run_error;
mod join_run_error_display;
mod join_state;
mod op_applied;
mod read_git_replay;
mod rebase_op;
mod replay;
mod replay_commit;
mod replay_fallback;
mod replay_git;
mod replay_in;
mod replay_result;
mod step;
mod with_join_lock;
mod write_join_state;

pub use join::join;
pub use join_answer::JoinAnswer;
pub use join_error::JoinError;
pub use join_run_error::JoinRunError;
pub use replay::replay;
pub use replay_result::ReplayResult;

#[cfg(test)]
mod after_replay_tests;
#[cfg(test)]
mod check_onto_tests;
#[cfg(test)]
mod decide_join_tests;
#[cfg(test)]
mod encode_join_state_tests;
#[cfg(test)]
mod fast_forward_op_tests;
#[cfg(test)]
mod fake_call;
#[cfg(test)]
mod join_answer_display_tests;
#[cfg(test)]
mod join_error_display_tests;
#[cfg(test)]
mod join_error_from_git_tests;
#[cfg(test)]
mod join_run_error_display_tests;
#[cfg(test)]
mod op_applied_tests;
#[cfg(test)]
mod read_git_replay_tests;
#[cfg(test)]
mod rebase_op_tests;
#[cfg(test)]
mod replay_commit_tests;
#[cfg(test)]
mod replay_fallback_tests;
#[cfg(test)]
mod replay_git_fake;
#[cfg(test)]
mod replay_git_fake_impl;
#[cfg(test)]
mod replay_in_tests;
