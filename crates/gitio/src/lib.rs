//! gitio: see the Mortar mission (Layout).
//! Past the 150-line limit because it is the crate's only bridge: one line per file of a flat crate, and nothing else.

mod absent_paths;
mod attr_check_request;
mod blob_bytes_request;
mod changed_files_request;
mod check_exit;
mod check_ref_format_request;
mod classify_update_refs;
mod commit_info;
mod commit_info_request;
mod commit_tree_request;
mod disk_usage_only_request;
mod folder_move_outcome;
mod folder_move_requests;
mod git;
mod git_attr_check;
mod git_blob_bytes;
mod git_changed_files;
mod git_check_ref_format;
mod git_command;
mod git_commit_info;
mod git_commit_tree;
mod git_common_dir;
mod git_dir_request;
mod git_disk_usage_only;
mod git_error;
mod git_error_display;
mod git_error_refusal;
mod git_folder_move;
mod git_git_dir;
mod git_head;
mod git_list_refs;
mod git_maintenance;
mod git_merge_base;
mod git_merge_tree;
mod git_new;
mod git_range_diff;
mod git_read_ref;
mod git_replay;
mod git_replay_available;
mod git_rev_list;
mod git_rev_parse;
mod git_run;
mod git_set_index;
mod git_snapshot_tree;
mod git_sparse_add;
mod git_sparse_list;
mod git_sparse_set;
mod git_status_paths;
mod git_toplevel;
mod git_trailers;
mod git_tree_entry;
mod git_tree_with_paths;
mod git_update_refs;
mod git_version;
mod git_with_trailers;
mod git_worktree_add;
mod git_worktree_list;
mod git_worktree_remove;
mod git_write_blob_paths;
mod head_requests;
mod index_info_stdin;
mod list_refs_request;
mod lossy_args;
mod maintenance_request;
mod merge_base_request;
mod merge_tree_outcome;
mod merge_tree_request;
mod next_temp_index;
mod no_nul;
mod oid;
mod oid_as_str;
mod oid_display;
mod oid_line;
mod oid_parse;
mod output;
mod path_from_bytes;
mod path_line;
mod person;
mod range_diff_request;
mod read_attr_check;
mod read_check_ref_format;
mod read_commit_info;
mod read_disk_usage;
mod read_folder_move;
mod read_merge_base;
mod read_merge_tree;
mod read_message;
mod read_nul_paths;
mod read_oid_lines;
mod read_path_entries;
mod read_ref_list;
mod read_ref_request;
mod read_replay;
mod read_replay_probe;
mod read_rev_parse;
mod read_show_ref;
mod read_sparse_list;
mod read_status_paths;
mod read_symbolic_ref;
mod read_trailers;
mod read_tree_entry;
mod read_worktree_list;
mod ref_update;
mod remove_on_drop;
mod replay_outcome;
mod replay_request;
mod rev_list_request;
mod rev_parse_request;
mod run_opts;
mod set_index_request;
mod settle;
mod snapshot_tree_requests;
mod sparse_add_request;
mod sparse_list_request;
mod sparse_set_requests;
mod spawn_refusal;
mod status_paths;
mod status_paths_request;
mod temp_index_path;
mod text_refusal;
mod trailers_add_request;
mod trailers_parse_request;
mod tree_entry_request;
mod tree_with_paths_list_request;
mod tree_with_paths_requests;
mod unquote_c_style;
mod update_refs_outcome;
mod update_refs_stdin;
mod usable_path;
mod usable_sparse_path;
mod usable_worktree_path;
mod version;
mod version_parse;
mod version_require_floor;
mod worktree;
mod worktree_add_request;
mod worktree_remove_request;
mod write_blob_paths_request;

pub use commit_info::CommitInfo;
pub use folder_move_outcome::FolderMoveOutcome;
pub use git::Git;
pub use git_error::GitError;
pub use merge_tree_outcome::MergeTreeOutcome;
pub use oid::Oid;
pub use output::Output;
pub use person::Person;
pub use ref_update::RefUpdate;
pub use replay_outcome::ReplayOutcome;
pub use run_opts::RunOpts;
pub use status_paths::StatusPaths;
pub use update_refs_outcome::UpdateRefsOutcome;
pub use version::Version;
pub use worktree::Worktree;

#[cfg(test)]
mod absent_paths_tests;
#[cfg(test)]
mod attr_check_request_tests;
#[cfg(test)]
mod blob_bytes_request_tests;
#[cfg(test)]
mod changed_files_request_tests;
#[cfg(test)]
mod check_exit_tests;
#[cfg(test)]
mod check_ref_format_request_tests;
#[cfg(test)]
mod classify_update_refs_tests;
#[cfg(test)]
mod commit_info_request_tests;
#[cfg(test)]
mod commit_tree_request_tests;
#[cfg(test)]
mod disk_usage_only_request_tests;
#[cfg(test)]
mod folder_move_requests_tests;
#[cfg(test)]
mod git_command_tests;
#[cfg(test)]
mod git_dir_request_tests;
#[cfg(test)]
mod git_disk_usage_only_tests;
#[cfg(test)]
mod git_error_display_tests;
#[cfg(test)]
mod git_error_refusal_tests;
#[cfg(test)]
mod git_error_tests;
#[cfg(test)]
mod git_new_tests;
#[cfg(test)]
mod git_run_tests;
#[cfg(test)]
mod git_tests;
#[cfg(test)]
mod git_version_tests;
#[cfg(test)]
mod head_requests_tests;
#[cfg(test)]
mod index_info_stdin_tests;
#[cfg(test)]
mod list_refs_request_tests;
#[cfg(test)]
mod lossy_args_tests;
#[cfg(test)]
mod maintenance_request_tests;
#[cfg(test)]
mod merge_base_request_tests;
#[cfg(test)]
mod merge_tree_request_tests;
#[cfg(test)]
mod next_temp_index_tests;
#[cfg(test)]
mod no_nul_tests;
#[cfg(test)]
mod oid_as_str_tests;
#[cfg(test)]
mod oid_display_tests;
#[cfg(test)]
mod oid_line_tests;
#[cfg(test)]
mod oid_parse_tests;
#[cfg(test)]
mod oid_tests;
#[cfg(test)]
mod output_tests;
#[cfg(test)]
mod path_from_bytes_tests;
#[cfg(test)]
mod path_line_tests;
#[cfg(test)]
mod range_diff_request_tests;
#[cfg(test)]
mod read_attr_check_tests;
#[cfg(test)]
mod read_check_ref_format_tests;
#[cfg(test)]
mod read_commit_info_tests;
#[cfg(test)]
mod read_disk_usage_tests;
#[cfg(test)]
mod read_folder_move_tests;
#[cfg(test)]
mod read_merge_base_tests;
#[cfg(test)]
mod read_merge_tree_tests;
#[cfg(test)]
mod read_message_tests;
#[cfg(test)]
mod read_nul_paths_tests;
#[cfg(test)]
mod read_oid_lines_tests;
#[cfg(test)]
mod read_path_entries_tests;
#[cfg(test)]
mod read_ref_list_tests;
#[cfg(test)]
mod read_ref_request_tests;
#[cfg(test)]
mod read_replay_probe_tests;
#[cfg(test)]
mod read_replay_tests;
#[cfg(test)]
mod read_rev_parse_tests;
#[cfg(test)]
mod read_show_ref_tests;
#[cfg(test)]
mod read_sparse_list_tests;
#[cfg(test)]
mod read_status_paths_tests;
#[cfg(test)]
mod read_symbolic_ref_tests;
#[cfg(test)]
mod read_trailers_tests;
#[cfg(test)]
mod read_tree_entry_tests;
#[cfg(test)]
mod read_worktree_list_tests;
#[cfg(test)]
mod replay_request_tests;
#[cfg(test)]
mod rev_list_request_tests;
#[cfg(test)]
mod rev_parse_request_tests;
#[cfg(test)]
mod run_opts_tests;
#[cfg(test)]
mod set_index_request_tests;
#[cfg(test)]
mod settle_tests;
#[cfg(test)]
mod snapshot_tree_requests_tests;
#[cfg(test)]
mod sparse_add_request_tests;
#[cfg(test)]
mod sparse_list_request_tests;
#[cfg(test)]
mod sparse_set_requests_tests;
#[cfg(test)]
mod spawn_refusal_tests;
#[cfg(test)]
mod status_paths_request_tests;
#[cfg(test)]
mod temp_index_path_tests;
#[cfg(test)]
mod text_refusal_tests;
#[cfg(test)]
mod trailers_add_request_tests;
#[cfg(test)]
mod trailers_parse_request_tests;
#[cfg(test)]
mod tree_entry_request_tests;
#[cfg(test)]
mod tree_with_paths_list_request_tests;
#[cfg(test)]
mod tree_with_paths_requests_tests;
#[cfg(test)]
mod unquote_c_style_tests;
#[cfg(test)]
mod update_refs_stdin_tests;
#[cfg(test)]
mod usable_path_tests;
#[cfg(test)]
mod usable_sparse_path_tests;
#[cfg(test)]
mod usable_worktree_path_tests;
#[cfg(test)]
mod version_parse_tests;
#[cfg(test)]
mod version_require_floor_tests;
#[cfg(test)]
mod version_tests;
#[cfg(test)]
mod worktree_add_request_tests;
#[cfg(test)]
mod worktree_remove_request_tests;
#[cfg(test)]
mod write_blob_paths_request_tests;
