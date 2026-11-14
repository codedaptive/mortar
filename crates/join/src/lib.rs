//! join: see the Mortar mission (Layout).

mod check_onto;
mod git_replay_git;
mod join_error;
mod join_error_display;
mod join_error_error;
mod join_error_from_git;
mod read_git_replay;
mod replay;
mod replay_commit;
mod replay_fallback;
mod replay_git;
mod replay_in;
mod replay_result;
mod step;

pub use join_error::JoinError;
pub use replay::replay;
pub use replay_result::ReplayResult;

#[cfg(test)]
mod check_onto_tests;
#[cfg(test)]
mod fake_call;
#[cfg(test)]
mod join_error_display_tests;
#[cfg(test)]
mod join_error_from_git_tests;
#[cfg(test)]
mod read_git_replay_tests;
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
