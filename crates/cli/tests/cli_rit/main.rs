//! RIT of the mortar child binary with folders, snapshot, join, gitio and oplog. Every case owns a disposable
//! repository.
//! It also holds the cli RITs moved by S-135 and S-136, owned by Nagatha from landing: each run_* module drives the
//! mortar binary against real git and the crates below cli.
mod json;
mod support;
mod saves;
mod joins;
mod recovery;
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
