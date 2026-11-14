//! Whether `mortar init` writes, has nothing to do, replaces or refuses.

use super::InitDecision;
use crate::config::Config;

/// No `existing` config: Write. Equal to `wanted`: AlreadyDone, with or without `replace`. Different: Replace
/// with `replace`, else Refused, each naming the keys that differ in the config's key order.
pub fn decide_init(existing: Option<&Config>, wanted: &Config, replace: bool) -> InitDecision {
    let Some(old) = existing else {
        return InitDecision::Write;
    };
    let keys = [
        ("origin_branch", old.origin_branch != wanted.origin_branch),
        ("integration_branch", old.integration_branch != wanted.integration_branch),
        ("remote", old.remote != wanted.remote),
        ("zones", old.zones != wanted.zones),
        ("root_files", old.root_files != wanted.root_files),
        ("build_dirs", old.build_dirs != wanted.build_dirs),
        ("cap_bytes", old.cap_bytes != wanted.cap_bytes),
        ("floor_bytes", old.floor_bytes != wanted.floor_bytes),
        ("formatter", old.formatter != wanted.formatter),
    ]
    .into_iter()
    .filter(|(_, differs)| *differs)
    .map(|(key, _)| key.to_string())
    .collect::<Vec<_>>();
    match (keys.is_empty(), replace) {
        (true, _) => InitDecision::AlreadyDone,
        (false, true) => InitDecision::Replace { keys },
        (false, false) => InitDecision::Refused { keys },
    }
}
