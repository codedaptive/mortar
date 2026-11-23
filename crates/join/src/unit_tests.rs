//! unit_tests: declares the crate's unit-test files. Test-only; kept out of lib.rs so lib.rs stays within 150
//! lines. Each test file reaches the code through crate:: paths.

#[path = "accumulate_paths_tests.rs"]
mod accumulate_paths_tests;
#[path = "after_replay_tests.rs"]
mod after_replay_tests;
#[path = "check_onto_tests.rs"]
mod check_onto_tests;
#[path = "classify_conflicted_tests.rs"]
mod classify_conflicted_tests;
#[path = "decide_base_join_tests.rs"]
mod decide_base_join_tests;
#[path = "decide_join_tests.rs"]
mod decide_join_tests;
#[path = "decide_resolve_tests.rs"]
mod decide_resolve_tests;
#[path = "decide_restore_tests.rs"]
mod decide_restore_tests;
#[path = "decode_join_state_tests.rs"]
mod decode_join_state_tests;
#[path = "encode_join_state_tests.rs"]
mod encode_join_state_tests;
#[path = "fast_forward_op_tests.rs"]
mod fast_forward_op_tests;
#[path = "file_markers_tests.rs"]
mod file_markers_tests;
#[path = "folder_tree_values_tests.rs"]
mod folder_tree_values_tests;
#[path = "join_answer_display_tests.rs"]
mod join_answer_display_tests;
#[path = "join_error_display_tests.rs"]
mod join_error_display_tests;
#[path = "join_error_from_git_tests.rs"]
mod join_error_from_git_tests;
#[path = "join_run_error_display_restore_tests.rs"]
mod join_run_error_display_restore_tests;
#[path = "join_run_error_display_tests.rs"]
mod join_run_error_display_tests;
#[path = "join_state_path_tests.rs"]
mod join_state_path_tests;
#[path = "join_state_value_tests.rs"]
mod join_state_value_tests;
#[path = "map_steps_tests.rs"]
mod map_steps_tests;
#[path = "marked_lines_tests.rs"]
mod marked_lines_tests;
#[path = "op_applied_tests.rs"]
mod op_applied_tests;
#[path = "pair_overlaps_tests.rs"]
mod pair_overlaps_tests;
#[path = "pair_overlaps_for_tests.rs"]
mod pair_overlaps_for_tests;
#[path = "read_git_replay_tests.rs"]
mod read_git_replay_tests;
#[path = "rebase_op_tests.rs"]
mod rebase_op_tests;
#[path = "recorded_join_state_tests.rs"]
mod recorded_join_state_tests;
#[path = "replay_commit_tests.rs"]
mod replay_commit_tests;
#[path = "replay_commits_tests.rs"]
mod replay_commits_tests;
#[path = "replay_fallback_tests.rs"]
mod replay_fallback_tests;
#[path = "replay_in_tests.rs"]
mod replay_in_tests;
#[path = "replay_mapped_in_tests.rs"]
mod replay_mapped_in_tests;
#[path = "resolved_commit_tests.rs"]
mod resolved_commit_tests;
#[path = "resolved_folder_tree_tests.rs"]
mod resolved_folder_tree_tests;
#[path = "restore_refusal_tests.rs"]
mod restore_refusal_tests;
#[path = "restore_state_step_tests.rs"]
mod restore_state_step_tests;
#[path = "tree_refusal_tests.rs"]
mod tree_refusal_tests;
#[path = "zone_request_op_tests.rs"]
mod zone_request_op_tests;
