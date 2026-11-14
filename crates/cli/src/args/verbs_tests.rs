//! VERBS: the built verbs and how each runs.

use super::{LockNeed, VerbKind, VERBS};

#[test]
fn only_init_is_built_and_it_is_a_harness_verb_without_lock_or_config() {
    assert_eq!(VERBS.len(), 1);
    let init = &VERBS[0];
    assert_eq!(init.name, "init");
    assert_eq!(init.kind, VerbKind::Harness);
    assert_eq!(init.lock, LockNeed::NoLock);
    assert!(!init.needs_config);
    assert!(init.usage.starts_with("mortar init --actor <name> --origin <branch> "));
    assert!(init.usage.ends_with(" [--replace] [--json]"));
}
