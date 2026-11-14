//! WO-orc-cli-rit-2 #1369: mortar child -> snapshot/join/folders/oplog, S-50/S-64/S-87/S-88/S-91.
//! WO-cli-rit-3 adds durable folder_tree refusal/retry, valued-op reads and resolve-kill -> next-join catch-up.
//! Each parent owns a throwaway repository. Status/need remain outside this target.
//! Existing binary fixture/JSON parser are read-only shared sources, not copies or concurrent CLI2s edits.
#[allow(dead_code)]
#[path = "../cli_rit/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../cli_rit/support.rs"]
mod support;
mod objects;
mod paused;
mod first_runs;
mod finishes;
mod unusual;
mod refusals;
mod pending;
mod catch_up;
