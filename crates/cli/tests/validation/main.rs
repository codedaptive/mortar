//! Validation through the built mortar and mr binaries, with folders, snapshot, join, land, gitio and oplog below
//! cli and real Git. Every case owns a disposable repository. The run_* modules drive one verb each; rit6..rit12
//! drive the binary across kits; guard_* cover the installed guard and its PATH lookup.
//! The fixture and JSON helpers live in tests/cli_rit and the observation fixtures in tests/operational_rit; both
//! are shared by path with the targets that also use them, never copied.
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
mod rit6;
mod rit7;
mod rit8;
mod rit9;
mod rit11;
mod rit12;
mod guard_path;
#[allow(dead_code)]
#[path = "../operational_rit/observe.rs"]
mod observe;
#[allow(dead_code)]
#[path = "../operational_rit/need_fixture.rs"]
mod need_fixture;
#[allow(dead_code)]
#[path = "../operational_rit/cid.rs"]
mod cid;
#[allow(dead_code)]
#[path = "../operational_rit/refs.rs"]
mod refs;
#[allow(dead_code)]
#[path = "../operational_rit/guard_fixture.rs"]
mod guard_fixture;
mod status_joined_tests;
mod run_need_tests;
mod run_guard_hook_tests;
mod status_attempt_folder_tests;
mod run_log_tests;
mod run_restore_tests;
mod run_restore_repeat_tests;
