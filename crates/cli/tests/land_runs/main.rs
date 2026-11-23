//! squash (with and without --message), withdraw, re-review, back-out, publish (with and without --no-push) and the
//! land flow and commit-files through the built mortar and mr binaries, against throwaway repositories with real Git.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod run_squash_tests;
mod run_rereview_tests;
mod run_rereview_savings_tests;
mod run_rereview_same_patch_tests;
mod run_back_out_tests;
mod run_publish_tests;
mod run_publish_no_push_tests;
mod land_flow;
mod run_withdraw_tests;
mod run_squash_message_tests;
mod run_commit_files_tests;
