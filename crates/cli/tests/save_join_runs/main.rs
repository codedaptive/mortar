//! save, join, recovery and process handling through the built mortar and mr binaries, against throwaway
//! repositories with real Git.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod saves;
mod joins;
mod exit_codes_tests;
mod partner_join_tests;
mod plain_base_tests;
mod zone_moved_sideways_tests;
mod recovery;
mod processes;
