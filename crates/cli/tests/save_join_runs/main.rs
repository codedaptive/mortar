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
mod recovery;
mod processes;
