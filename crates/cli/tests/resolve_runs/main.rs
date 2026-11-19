//! resolve and the next join after it through the built mortar and mr binaries, against throwaway repositories with
//! real Git. objects and paused are the paused-join fixtures.
#[allow(dead_code)]
#[path = "../common/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../common/support/mod.rs"]
mod support;
mod resolve_first_runs;
mod resolve_finishes;
mod resolve_unusual;
mod resolve_refusals;
mod resolve_pending;
mod resolve_catch_up;
mod objects;
mod paused;
