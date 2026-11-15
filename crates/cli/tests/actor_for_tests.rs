//! actor_for: agent verbs act as "<zone>/<request>" and refuse --actor; harness verbs require --actor; hook verbs
//! refuse --actor. A hook verb acting as "git" is the ZUT in src/context/actor_for_tests.rs.
//! ZIT (S-79 as replaced: the line is the file): it calls args/usage_report for real.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use cli::{actor_for, ExitCode, LockNeed, VerbKind, VerbSpec, VERBS};

const AGENT: VerbSpec = VerbSpec {
    name: "save",
    kind: VerbKind::Agent,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mortar save [--json]",
};

const HOOK: VerbSpec = VerbSpec {
    name: "guard-hook",
    kind: VerbKind::Hook,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mortar guard-hook <hook>",
};

fn init() -> &'static VerbSpec {
    VERBS.iter().find(|verb| verb.name == "init").expect("init is in the table")
}

fn record(request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind: if request.is_some() { FolderKind::Request } else { FolderKind::Zone },
        zone: "orc".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: request.map(|_| "zone/orc".to_string()),
    }
}

#[test]
fn an_agent_verb_acts_as_its_request() {
    assert_eq!(actor_for(&AGENT, Some(&record(Some("WO-cli-1"))), None), Ok("orc/WO-cli-1".to_string()));
}

#[test]
fn an_agent_verb_refuses_actor_as_usage() {
    let report = actor_for(&AGENT, Some(&record(Some("WO-cli-1"))), Some("bob")).expect_err("usage");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "save");
    assert_eq!(report.lines[0], "agent verbs take the actor from the folder record");
    assert_eq!(report.next, "mortar save [--json]");
}

#[test]
fn an_agent_verb_outside_a_request_folder_is_refused_naming_the_folder() {
    let zone = record(None);
    let cases: [(Option<&FolderRecord>, &str); 2] = [
        (Some(&zone), "this is a zone folder (zone orc): agent verbs run in a request folder"),
        (None, "this directory is not in a Mortar folder"),
    ];
    for (folder, why) in cases {
        let report = actor_for(&AGENT, folder, None).expect_err("refused");
        assert_eq!(report.exit as i32, 1);
        assert_eq!(report.verb, "save");
        assert_eq!(report.lines, vec![why.to_string(), "nothing ran".to_string()]);
        assert_eq!(report.next, "run mortar save inside a request folder Mortar made");
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
    assert_eq!(report.exit as i32, 4);
    assert_eq!(report.verb, "init");
    assert_eq!(report.lines, vec!["--actor <name> is required".to_string(), "verbs: guard-hook, init, join, log, need, resolve, restore, save, status".to_string()]);
    assert_eq!(report.next, init().usage);
}

#[test]
fn a_harness_verb_with_an_empty_actor_is_usage() {
    assert_eq!(actor_for(init(), None, Some("")).expect_err("usage").exit, ExitCode::Usage);
}

#[test]
fn a_hook_verb_refuses_actor_as_usage() {
    let report = actor_for(&HOOK, None, Some("bob")).expect_err("usage");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.verb, "guard-hook");
    assert_eq!(report.lines[0], "hook verbs are run by git and take no --actor");
    assert_eq!(report.next, "mortar guard-hook <hook>");
}
