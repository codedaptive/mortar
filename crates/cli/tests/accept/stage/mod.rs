//! Step 1's set-up, shared by steps 1-4 (stage_new), and the helpers the steps run Mortar and git with: a
//! harness verb as the drive's actor, an agent verb in a request folder, a request folder made, an edit saved or
//! joined, a git read (of the working repository, of the bare remote, of whether one commit holds another), the
//! reply read from `--json`, a set-up line recorded, and whether a file holds conflict markers. Each helper returns
//! what it saw, so the caller records it. Two test-only builders serve the steps' tests: test_ran and failed_line.

mod agent_mr;
mod git_out;
mod harness_mr;
mod holds_conflict_markers;
mod is_ancestor;
mod read_reply;
mod remote_git;
mod reply;
mod reply_is;
mod reply_text;
mod reply_texts;
mod setup_step;
mod stage;
mod stage_edit;
mod stage_join;
mod stage_new;
mod stage_request;
mod stage_save;

#[cfg(test)]
mod failed_line;
#[cfg(test)]
mod holds_conflict_markers_tests;
#[cfg(test)]
mod read_reply_tests;
#[cfg(test)]
mod reply_is_tests;
#[cfg(test)]
mod reply_text_tests;
#[cfg(test)]
mod reply_texts_tests;
#[cfg(test)]
mod setup_step_tests;
#[cfg(test)]
mod test_ran;

pub use agent_mr::agent_mr;
pub use git_out::git_out;
#[cfg(test)]
pub use failed_line::failed_line;
pub use harness_mr::harness_mr;
pub use holds_conflict_markers::holds_conflict_markers;
pub use is_ancestor::is_ancestor;
pub use read_reply::read_reply;
pub use remote_git::remote_git;
pub use reply::Reply;
pub use setup_step::setup_step;
pub use stage::Stage;
pub use stage_edit::stage_edit;
pub use stage_join::stage_join;
pub use stage_new::stage_new;
pub use stage_request::stage_request;
pub use stage_save::stage_save;
#[cfg(test)]
pub use test_ran::test_ran;
