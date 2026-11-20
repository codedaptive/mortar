//! squash, re-review, back-out, publish and the land flow through the built mortar and mr binaries, against
//! throwaway repositories with real Git.
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
mod land_flow;
