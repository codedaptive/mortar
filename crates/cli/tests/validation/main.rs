//! Validation through the built mortar and mr binaries, with folders, snapshot, join, land, gitio and oplog below
//! cli and real Git. Every case owns a disposable repository. The run_* modules drive one verb each; land_flow,
//! bisect_backup, after_publish, snapshot_overlap_disk_sweep and undo_save_join_resolve drive the binary across kits;
//! status, need, guard and handoff drive status, need and the installed guard hooks; resolve_* drive resolve and
//! the next join after it; guard_* cover the installed guard and its PATH lookup.
//! The fixture and JSON helpers live in tests/cli_rit and are shared by path, never copied; the observation, need,
//! guard and paused-join fixtures (observe, need_fixture, cid, refs, guard_fixture, objects, paused) live here.
#[path = "../cli_rit/json.rs"]
mod json;
#[path = "../cli_rit/support.rs"]
mod support;
mod guard_linked_worktree;
mod guard_symref;
mod saves;
mod joins;
mod recovery;
mod precheck_status;
mod processes;
mod run_folder_zone_tests;
mod run_folder_request_tests;
mod run_folder_remove_tests;
mod run_folder_attempts_tests;
mod run_folder_keep_tests;
mod run_folder_drop_tests;
mod run_guard_install_tests;
mod run_guard_status_tests;
mod run_squash_tests;
mod run_rereview_tests;
mod run_back_out_tests;
mod run_publish_tests;
mod land_flow;
mod bisect_backup;
mod after_publish;
mod snapshot_overlap_disk_sweep;
mod undo_save_join_resolve;
mod guard_install_path_lookup;
mod guard_path;
#[allow(dead_code)]
mod observe;
#[allow(dead_code)]
mod need_fixture;
#[allow(dead_code)]
mod cid;
#[allow(dead_code)]
mod refs;
#[allow(dead_code)]
mod guard_fixture;
mod objects;
mod paused;
mod status_joined_tests;
mod run_need_tests;
mod run_guard_hook_tests;
mod status_attempt_folder_tests;
mod run_log_tests;
mod run_restore_tests;
mod run_restore_repeat_tests;
mod status;
mod need;
mod guard;
mod handoff;
mod resolve_first_runs;
mod resolve_finishes;
mod resolve_unusual;
mod resolve_refusals;
mod resolve_pending;
mod resolve_catch_up;
mod run_git_floor_tests;
