//! join: see the Mortar mission (Layout).

mod accumulate_paths;
mod after_replay;
mod after_replay_step;
mod check_onto;
mod classify_conflicted;
mod conflicted_entry;
mod continue_replay;
mod decide_join;
mod decide_resolve;
mod decode_join_state;
mod encode_join_state;
mod fast_forward_op;
mod file_markers;
mod folder_tree_name;
mod folder_tree_values;
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
mod join_state_path;
mod marked_file;
mod marked_lines;
mod op_applied;
mod open_request;
mod overlap;
mod overlap_pair;
mod pair_overlaps;
mod pause;
mod paused_join;
mod paused_join_type;
mod read_git_replay;
mod read_join_state;
mod read_marked_files;
mod rebase_op;
mod remove_join_state;
mod replay;
mod replay_commit;
mod replay_commits;
mod replay_fallback;
mod replay_git;
mod replay_in;
mod replay_result;
mod resolve;
mod resolve_step;
mod resolved_commit;
mod resolved_folder_tree;
mod save_join_state;
mod step;
mod tree_refusal;
mod write_conflicted_files;
mod write_join_state;

pub use folder_tree_name::FOLDER_TREE;
pub use join::join;
pub use join_answer::JoinAnswer;
pub use join_error::JoinError;
pub use join_run_error::JoinRunError;
pub use marked_file::MarkedFile;
pub use open_request::OpenRequest;
pub use overlap::overlap;
pub use overlap_pair::OverlapPair;
pub use paused_join::paused_join;
pub use paused_join_type::PausedJoin;
pub use replay::replay;
pub use replay_result::ReplayResult;
pub use resolve::resolve;

#[cfg(test)]
mod accumulate_paths_tests;
#[cfg(test)]
mod after_replay_tests;
#[cfg(test)]
mod check_onto_tests;
#[cfg(test)]
mod classify_conflicted_tests;
#[cfg(test)]
mod decide_join_tests;
#[cfg(test)]
mod decide_resolve_tests;
#[cfg(test)]
mod decode_join_state_tests;
#[cfg(test)]
mod encode_join_state_tests;
#[cfg(test)]
mod fake_call;
#[cfg(test)]
mod fast_forward_op_tests;
#[cfg(test)]
mod file_markers_tests;
#[cfg(test)]
mod folder_tree_values_tests;
#[cfg(test)]
mod join_answer_display_tests;
#[cfg(test)]
mod join_error_display_tests;
#[cfg(test)]
mod join_error_from_git_tests;
#[cfg(test)]
mod join_run_error_display_tests;
#[cfg(test)]
mod join_state_path_tests;
#[cfg(test)]
mod marked_lines_tests;
#[cfg(test)]
mod op_applied_tests;
#[cfg(test)]
mod pair_overlaps_tests;
#[cfg(test)]
mod read_git_replay_tests;
#[cfg(test)]
mod rebase_op_tests;
#[cfg(test)]
mod replay_commit_tests;
#[cfg(test)]
mod replay_commits_tests;
#[cfg(test)]
mod replay_fallback_tests;
#[cfg(test)]
mod replay_git_fake;
#[cfg(test)]
mod replay_git_fake_impl;
#[cfg(test)]
mod replay_in_tests;
#[cfg(test)]
mod resolved_commit_tests;
#[cfg(test)]
mod resolved_folder_tree_tests;
#[cfg(test)]
mod tree_refusal_tests;
