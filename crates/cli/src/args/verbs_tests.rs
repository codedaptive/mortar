//! VERBS: the built verbs and how each runs.

use super::{LockNeed, VerbKind, VerbSpec, VERBS};

fn verb(name: &str) -> &'static VerbSpec {
    VERBS.iter().find(|spec| spec.name == name).expect("the verb is in the table")
}

#[test]
fn built_verbs_are_listed_alphabetically() {
    let names: Vec<&str> = VERBS.iter().map(|verb| verb.name).collect();
    assert_eq!(
        names,
        vec![
            "after-publish",
            "back-out",
            "backup",
            "bisect",
            "disk",
            "folder",
            "guard",
            "guard-hook",
            "init",
            "join",
            "land",
            "log",
            "need",
            "overlap",
            "publish",
            "rereview",
            "resolve",
            "restore",
            "restore-backup",
            "save",
            "snapshot",
            "squash",
            "status",
            "sweep",
            "would-merge"
        ]
    );
}

#[test]
fn after_publish_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let after_publish = verb("after-publish");
    assert_eq!(after_publish.kind, VerbKind::Harness);
    assert_eq!(after_publish.lock, LockNeed::Land);
    assert!(after_publish.needs_config);
    assert_eq!(
        after_publish.usage,
        "mortar after-publish <zone> [--move <zone>]... --actor <name> [--key <operation key>] [--json]"
    );
}

#[test]
fn snapshot_is_a_harness_verb_under_the_join_lock_that_needs_the_config() {
    let snapshot = verb("snapshot");
    assert_eq!(snapshot.kind, VerbKind::Harness);
    assert_eq!(snapshot.lock, LockNeed::Join);
    assert!(snapshot.needs_config);
    assert_eq!(snapshot.usage, "mortar snapshot --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn overlap_is_a_harness_verb_without_lock_that_needs_the_config() {
    let overlap = verb("overlap");
    assert_eq!(overlap.kind, VerbKind::Harness);
    assert_eq!(overlap.lock, LockNeed::NoLock);
    assert!(overlap.needs_config);
    assert_eq!(overlap.usage, "mortar overlap <zone> --actor <name> [--json]");
}

#[test]
fn disk_is_a_harness_verb_without_lock_that_needs_the_config() {
    let disk = verb("disk");
    assert_eq!(disk.kind, VerbKind::Harness);
    assert_eq!(disk.lock, LockNeed::NoLock);
    assert!(disk.needs_config);
    assert_eq!(disk.usage, "mortar disk --actor <name> [--json]");
}

#[test]
fn sweep_is_a_harness_verb_under_the_folders_lock_that_needs_the_config() {
    let sweep = verb("sweep");
    assert_eq!(sweep.kind, VerbKind::Harness);
    assert_eq!(sweep.lock, LockNeed::Folders);
    assert!(sweep.needs_config);
    assert_eq!(
        sweep.usage,
        "mortar sweep [--idle-days <n>] [--ref <full ref name>]... --actor <name> [--key <operation key>] [--json]"
    );
}

#[test]
fn back_out_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let back_out = verb("back-out");
    assert_eq!(back_out.kind, VerbKind::Harness);
    assert_eq!(back_out.lock, LockNeed::Land);
    assert!(back_out.needs_config);
    assert_eq!(back_out.usage, "mortar back-out --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn publish_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let publish = verb("publish");
    assert_eq!(publish.kind, VerbKind::Harness);
    assert_eq!(publish.lock, LockNeed::Land);
    assert!(publish.needs_config);
    assert_eq!(publish.usage, "mortar publish --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn would_merge_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let would_merge = verb("would-merge");
    assert_eq!(would_merge.kind, VerbKind::Harness);
    assert_eq!(would_merge.lock, LockNeed::Land);
    assert!(would_merge.needs_config);
    assert_eq!(
        would_merge.usage,
        "mortar would-merge <commit or change ID> --actor <name> [--key <operation key>] [--json]"
    );
}

#[test]
fn land_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let land = verb("land");
    assert_eq!(land.kind, VerbKind::Harness);
    assert_eq!(land.lock, LockNeed::Land);
    assert!(land.needs_config);
    assert_eq!(land.usage, "mortar land <commit> --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn rereview_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let rereview = verb("rereview");
    assert_eq!(rereview.kind, VerbKind::Harness);
    assert_eq!(rereview.lock, LockNeed::Land);
    assert!(rereview.needs_config);
    assert_eq!(
        rereview.usage,
        "mortar rereview <reviewed commit> <new commit> --actor <name> [--key <operation key>] [--json]"
    );
}

#[test]
fn squash_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let squash = verb("squash");
    assert_eq!(squash.kind, VerbKind::Harness);
    assert_eq!(squash.lock, LockNeed::Land);
    assert!(squash.needs_config);
    assert_eq!(squash.usage, "mortar squash <zone> --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn guard_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let guard = verb("guard");
    assert_eq!(guard.kind, VerbKind::Harness);
    assert_eq!(guard.lock, LockNeed::Land);
    assert!(guard.needs_config);
    assert_eq!(guard.usage, "mortar guard (install | status) --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn folder_is_a_harness_verb_under_the_join_lock_that_needs_the_config() {
    let folder = verb("folder");
    assert_eq!(folder.kind, VerbKind::Harness);
    assert_eq!(folder.lock, LockNeed::Join);
    assert!(folder.needs_config);
    assert_eq!(
        folder.usage,
        "mortar folder (zone <zone> <path> [--start <commit>] | request <zone> <request> <path> --paths <path>... | \
         remove <path> | attempts <n> <dir> | keep <k> | drop) --actor <name> [--key <operation key>] [--json]"
    );
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

#[test]
fn log_is_a_read_only_verb_without_lock_that_needs_the_config() {
    let log = verb("log");
    assert_eq!(log.kind, VerbKind::ReadOnly);
    assert_eq!(log.lock, LockNeed::NoLock);
    assert!(log.needs_config);
    assert_eq!(log.usage, "mortar log [--ref <full ref name>] [--actor <name>] [--key <operation key>] [--json]");
}

#[test]
fn restore_is_a_harness_verb_whose_lock_comes_from_its_refs_that_needs_the_config() {
    let restore = verb("restore");
    assert_eq!(restore.kind, VerbKind::Harness);
    assert_eq!(restore.lock, LockNeed::FromRefs);
    assert!(restore.needs_config);
    assert_eq!(
        restore.usage,
        "mortar restore <op id> <full ref name>... --actor <name> [--key <operation key>] [--json]"
    );
}

#[test]
fn backup_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let backup = verb("backup");
    assert_eq!(backup.kind, VerbKind::Harness);
    assert_eq!(backup.lock, LockNeed::Land);
    assert!(backup.needs_config);
    assert_eq!(backup.usage, "mortar backup --actor <name> [--key <operation key>] [--json]");
}

#[test]
fn bisect_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let bisect = verb("bisect");
    assert_eq!(bisect.kind, VerbKind::Harness);
    assert_eq!(bisect.lock, LockNeed::Land);
    assert!(bisect.needs_config);
    assert_eq!(
        bisect.usage,
        "mortar bisect <last pass> <failed> --timeout <seconds> --actor <name> [--key <operation key>] [--json] -- \
         <command>..."
    );
}

#[test]
fn restore_backup_is_a_harness_verb_under_the_land_lock_that_needs_the_config() {
    let restore_backup = verb("restore-backup");
    assert_eq!(restore_backup.kind, VerbKind::Harness);
    assert_eq!(restore_backup.lock, LockNeed::Land);
    assert!(restore_backup.needs_config);
    assert_eq!(
        restore_backup.usage,
        "mortar restore-backup <host> --actor <name> [--key <operation key>] [--json]"
    );
}
