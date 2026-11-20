//! status, need and handoff through the built mortar and mr binaries, against throwaway repositories with real Git.
//! observe and need_fixture are the observation and need fixtures.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod savings;
mod savings_baseline;
mod status;
mod need;
mod handoff;
mod status_joined_tests;
mod status_attempt_folder_tests;
mod run_need_tests;
mod precheck_status;
#[allow(dead_code)]
mod observe;
#[allow(dead_code)]
mod need_fixture;
