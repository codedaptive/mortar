//! actor_for: a hook verb with no --actor acts as "git" and refuses --actor; a read-only verb acts as nobody with or
//! without --actor; a setup verb acts as nobody and refuses --actor; agent verbs act as "<zone>/<request>" and
//! refuse --actor; harness verbs require --actor.
//! Unit tests: besides actor_for it uses the args verb types and the folder record, which are plain data, and
//! args/usage_report and the verb table, which are pure.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::actor_for;
use crate::args::{LockNeed, VerbKind, VerbSpec, VERBS};
use crate::report::ExitCode;

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
    assert_eq!(actor_for(&READ_ONLY, None, Some("agent-a")), Ok(String::new()));
}

const AGENT: VerbSpec = VerbSpec {
    name: "save",
    kind: VerbKind::Agent,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mr save [--json]",
};

fn init() -> &'static VerbSpec {
    VERBS.iter().find(|verb| verb.name == "init").expect("init is in the table")
}

fn record(request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind: if request.is_some() { FolderKind::Request } else { FolderKind::Zone },
        zone: "a".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: request.map(|_| "zone/a".to_string()),
    }
}

#[test]
fn an_agent_verb_acts_as_its_request() {
    assert_eq!(actor_for(&AGENT, Some(&record(Some("r1"))), None), Ok("a/r1".to_string()));
}

#[test]
fn an_agent_verb_refuses_actor_as_usage() {
    let report = actor_for(&AGENT, Some(&record(Some("r1"))), Some("agent-a")).expect_err("usage");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "save");
    assert_eq!(report.lines[0], "agent verbs take the actor from the folder record");
    assert_eq!(report.next, "mr save [--json]");
}

#[test]
fn an_agent_verb_outside_a_request_folder_is_refused_naming_the_folder() {
    let zone = record(None);
    let cases: [(Option<&FolderRecord>, &str); 2] = [
        (Some(&zone), "this is a zone folder (zone a): agent verbs run in a request folder"),
        (None, "this directory is not in a Mortar folder"),
    ];
    for (folder, why) in cases {
        let report = actor_for(&AGENT, folder, None).expect_err("refused");
        assert_eq!(report.exit as i32, 3);
        assert_eq!(report.verb, "save");
        assert_eq!(report.lines, vec![why.to_string(), "nothing ran".to_string()]);
        assert_eq!(report.next, "run mr save inside a request folder Mortar made");
    }
}

#[test]
fn a_harness_verb_acts_as_actor() {
    assert_eq!(actor_for(init(), None, Some("harness-1")), Ok("harness-1".to_string()));
}

#[test]
fn a_harness_verb_ignores_the_record() {
    assert_eq!(actor_for(init(), Some(&record(Some("r"))), Some("h")), Ok("h".to_string()));
}

#[test]
fn a_harness_verb_without_actor_is_usage() {
    let report = actor_for(init(), None, None).expect_err("usage");
    assert_eq!(report.exit as i32, 2);
    assert_eq!(report.verb, "init");
    assert_eq!(report.lines, vec!["--actor <name> is required".to_string(), "verbs: after-publish, back-out, backup, bisect, commit-files, disk, folder, guard, guard-hook, help, init, install, join, land, log, need, overlap, publish, rereview, resolve, restore, restore-backup, save, snapshot, squash, status, sweep, undo, uninstall, withdraw, would-merge".to_string()]);
    assert_eq!(report.next, init().usage);
}

#[test]
fn a_harness_verb_with_an_empty_actor_is_usage() {
    assert_eq!(actor_for(init(), None, Some("")).expect_err("usage").exit, ExitCode::Usage);
}

#[test]
fn a_hook_verb_refuses_actor_as_usage() {
    let report = actor_for(&HOOK, None, Some("agent-a")).expect_err("usage");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "guard-hook");
    assert_eq!(report.lines[0], "hook verbs are run by git and take no --actor");
    assert_eq!(report.next, "mortar guard-hook <hook>");
}

const SETUP: VerbSpec = VerbSpec {
    name: "install",
    kind: VerbKind::Setup,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mortar install [--json]",
};

#[test]
fn a_setup_verb_acts_as_nobody() {
    assert_eq!(actor_for(&SETUP, None, None), Ok(String::new()));
}

#[test]
fn a_setup_verb_refuses_actor_as_usage() {
    let report = actor_for(&SETUP, None, Some("agent-a")).expect_err("usage");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "install");
    assert_eq!(report.lines[0], "install and uninstall take no --actor");
    assert_eq!(report.next, "mortar install [--json]");
}
