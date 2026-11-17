//! actor_for: a hook verb with no --actor acts as "git"; a read-only verb acts as nobody with or without --actor.
//! ZUT: besides actor_for it uses only the args verb types, which are plain data.

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

const READ_ONLY: VerbSpec = VerbSpec {
    name: "log",
    kind: VerbKind::ReadOnly,
    lock: LockNeed::NoLock,
    needs_config: true,
    usage: "mr log [--actor <name>]",
};

#[test]
fn a_read_only_verb_acts_as_nobody_with_and_without_actor() {
    assert_eq!(actor_for(&READ_ONLY, None, None), Ok(String::new()));
    assert_eq!(actor_for(&READ_ONLY, None, Some("bob")), Ok(String::new()));
}
