//! VERBS: the built verbs and how each runs.

use super::{LockNeed, VerbKind, VerbSpec, VERBS};

fn verb(name: &str) -> &'static VerbSpec {
    VERBS.iter().find(|spec| spec.name == name).expect("the verb is in the table")
}

#[test]
fn built_verbs_are_listed_alphabetically() {
    let names: Vec<&str> = VERBS.iter().map(|verb| verb.name).collect();
    assert_eq!(names, vec!["guard-hook", "init", "join", "need", "resolve", "save", "status"]);
}

#[test]
fn guard_hook_is_a_hook_verb_without_lock_or_config() {
    let guard_hook = verb("guard-hook");
    assert_eq!(guard_hook.kind, VerbKind::Hook);
    assert_eq!(guard_hook.lock, LockNeed::NoLock);
    assert!(!guard_hook.needs_config);
    assert_eq!(guard_hook.usage, "mortar guard-hook (reference-transaction <state> | pre-push <remote> <url>)");
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

#[test]
fn need_is_an_agent_verb_under_the_join_lock_that_needs_the_config() {
    let need = verb("need");
    assert_eq!(need.kind, VerbKind::Agent);
    assert_eq!(need.lock, LockNeed::Join);
    assert!(need.needs_config);
    assert_eq!(need.usage, "mortar need <path> [--key <key>] [--json]");
}

#[test]
fn status_is_an_agent_verb_without_lock_that_needs_the_config() {
    let status = verb("status");
    assert_eq!(status.kind, VerbKind::Agent);
    assert_eq!(status.lock, LockNeed::NoLock);
    assert!(status.needs_config);
    assert_eq!(status.usage, "mortar status [--json]");
}
