//! VERBS: the built verbs and how each runs.

use super::{LockNeed, VerbKind, VerbSpec, VERBS};

fn verb(name: &str) -> &'static VerbSpec {
    VERBS.iter().find(|spec| spec.name == name).expect("the verb is in the table")
}

#[test]
fn init_join_resolve_and_save_are_built_in_that_order() {
    let names: Vec<&str> = VERBS.iter().map(|verb| verb.name).collect();
    assert_eq!(names, vec!["init", "join", "resolve", "save"]);
}

#[test]
fn init_is_a_harness_verb_without_lock_or_config() {
    let init = verb("init");
    assert_eq!(init.kind, VerbKind::Harness);
    assert_eq!(init.lock, LockNeed::NoLock);
    assert!(!init.needs_config);
    assert!(init.usage.starts_with("mortar init --actor <name> --origin <branch> "));
    assert!(init.usage.ends_with(" [--replace] [--json]"));
}

#[test]
fn join_is_an_agent_verb_under_the_join_lock_that_needs_the_config() {
    let join = verb("join");
    assert_eq!(join.kind, VerbKind::Agent);
    assert_eq!(join.lock, LockNeed::Join);
    assert!(join.needs_config);
    assert_eq!(join.usage, "mortar join [--key <operation key>] [--json]");
}

#[test]
fn resolve_is_an_agent_verb_under_the_join_lock_that_needs_the_config() {
    let resolve = verb("resolve");
    assert_eq!(resolve.kind, VerbKind::Agent);
    assert_eq!(resolve.lock, LockNeed::Join);
    assert!(resolve.needs_config);
    assert_eq!(resolve.usage, "mortar resolve [--key <operation key>] [--json]");
}

#[test]
fn save_is_an_agent_verb_under_the_join_lock_that_needs_the_config() {
    let save = verb("save");
    assert_eq!(save.kind, VerbKind::Agent);
    assert_eq!(save.lock, LockNeed::Join);
    assert!(save.needs_config);
    assert!(save.usage.starts_with("mortar save \"<sentence>\""));
}
