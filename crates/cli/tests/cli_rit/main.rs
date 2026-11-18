//! RIT of the mortar child binary with folders, snapshot, join, gitio and oplog. Every case owns a disposable
//! repository, except the cases of restore_git_errors.
//! It also holds the cli RITs moved by S-135 and S-136, owned by Nagatha from landing: each run_* module drives the
//! mortar binary against real git and the crates below cli. restore_git_errors calls cli's restore_why,
//! restore_check_report and restore_failed_report directly, with gitio's GitError Display run for real: no binary, no
//! repository.
mod json;
mod support;
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
mod guard_symref;
mod restore_git_errors;
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
