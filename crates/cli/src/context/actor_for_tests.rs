//! actor_for: a hook verb with no --actor acts as "git". ZUT: besides actor_for it uses only the args verb types,
//! which are plain data.

use super::actor_for;
use crate::args::{LockNeed, VerbKind, VerbSpec};

const HOOK: VerbSpec = VerbSpec {
    name: "guard-hook",
    kind: VerbKind::Hook,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mortar guard-hook <hook>",
};

#[test]
fn a_hook_verb_acts_as_git() {
    assert_eq!(actor_for(&HOOK, None, None), Ok("git".to_string()));
}
