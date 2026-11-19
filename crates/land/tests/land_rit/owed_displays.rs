//! Land -> Gitio/Oplog Display seams, with independent literal expectations.
//! Durations below are static values being formatted, not measurements or timeout executions.
use gitio::{GitError, Oid};
use land::{BackupError, BisectAnswer, BisectError, BisectReport, Culprit, CulpritError, GuardError,
    LandError, Probe, ProbeOutcome, PublishError, RereviewError, SquashError};
use oplog::{ChangeId, LockError, OplogError};
use std::{path::PathBuf, time::Duration};
use testkit::TestRepo;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn id() -> ChangeId { ChangeId::parse("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk").unwrap() }
fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}
fn probe(outcome: ProbeOutcome) -> Probe { Probe { commit: a(), outcome, folder: PathBuf::from("unused-probe-folder") } }
fn report(change_id: Option<ChangeId>, zone: Option<&str>) -> BisectReport {
    BisectReport { answer: BisectAnswer::Culprit(Culprit { commit: a(), change_id, zone: zone.map(str::to_string) }), probes: vec![] }
}
macro_rules! case {
    ($name:ident, $value:expr, $expected:literal) => {
        #[test] fn $name() { let _repo = TestRepo::new().unwrap(); assert_eq!(($value).to_string(), $expected); }
    };
}

case!(rit_host_says_nothing_moved, BackupError::Host(LockError::HostUnknown("hostname failed".into())),
    "backup host name: cannot read this host's name: hostname failed; check that `hostname` runs on this machine; nothing was pushed or restored");
case!(rit_git_is_marked_as_backups, BackupError::Git(git_error()),
    "backup git: git read-tree HEAD: exit code 128: fatal: no tree");
#[test]
fn rit_oplog_is_marked_as_backups() {
    let _repo = TestRepo::new().unwrap();
    let shown = BackupError::Oplog(OplogError::UnknownOp { op_id: "op-404".into() }).to_string();
    assert!(shown == "backup op log: cannot restore: op op-404 is not in the op log; find the op id with mortar log"
        || shown == "backup op log: cannot restore: op op-404 is not in the op log; find the op id with mr log", "{shown}");
}
case!(rit_git_text_starts_with_git_failed, GuardError::Git(git_error()),
    "git failed: git read-tree HEAD: exit code 128: fatal: no tree");
case!(rit_not_on_origin_names_the_commit_and_the_branch, BisectError::NotOnOrigin { failed: a(), origin_branch: "main".into() },
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not on main; give a failed commit that landed on main");
case!(rit_off_the_line_names_both_commits, BisectError::OffTheLine { last_pass: a(), failed: b() },
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not on bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb's first-parent line, so the range names no landings; give a last passing commit that is a landing before bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
case!(rit_probe_could_not_run_names_the_commit_and_the_error, BisectError::ProbeCouldNotRun { commit: a(), error: "spawn refused".into() },
    "the test command could not be run at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: spawn refused; fix the command, then bisect again");
case!(rit_a_pass_reads_passed, probe(ProbeOutcome::Passed), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: passed");
case!(rit_a_fail_names_its_exit_code, probe(ProbeOutcome::Failed { code: Some(1) }), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed with exit code 1");
case!(rit_a_signal_kill_says_so, probe(ProbeOutcome::Failed { code: None }), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, killed by a signal");
case!(rit_a_timeout_says_timed_out_with_whole_seconds, probe(ProbeOutcome::TimedOut { limit: Duration::from_secs(60) }),
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 60s");
case!(rit_a_timeout_of_part_seconds_reads_in_milliseconds, probe(ProbeOutcome::TimedOut { limit: Duration::from_millis(1500) }),
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 1500ms");
case!(rit_a_culprit_with_trailers_names_its_commit_change_and_zone, report(Some(id()), Some("zone-x")),
    "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, zone zone-x");
case!(rit_a_culprit_without_trailers_says_it_carries_none, report(None, None),
    "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; it carries no squash trailers");
case!(rit_a_culprit_without_a_zone_names_the_missing_zone_trailer, report(Some(id()), None),
    "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk; it carries no Mortar-Zone trailer");
case!(rit_a_culprit_without_a_change_id_names_the_missing_change_id_trailer, report(None, Some("zone-x")),
    "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, zone zone-x; it carries no Mortar-Change-Id trailer");

#[test]
fn rit_probes_follow_the_answer_indented_in_the_order_run() {
    let _repo = TestRepo::new().unwrap(); let mut report = report(Some(id()), Some("zone-x"));
    report.probes = vec![probe(ProbeOutcome::TimedOut { limit: Duration::from_secs(30) }),
        Probe { commit: b(), outcome: ProbeOutcome::Passed, folder: PathBuf::from("unused-second-folder") }];
    assert_eq!(report.to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, zone zone-x\n  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 30s\n  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: passed");
}

case!(rit_no_zone_names_the_commit_and_says_nothing_moved, LandError::NoZone { commit: a() },
    "squash commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries no Mortar-Zone trailer, so its landing cannot name the zone; nothing was moved");
case!(rit_no_landing_state_names_both_tips_and_says_nothing_moved, LandError::NoLandingState { origin: a(), integration: b() },
    "integration is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, not at origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, and no landing state names a landing in flight; nothing was moved");
case!(rit_integration_elsewhere_names_both_commits_and_says_nothing_moved, LandError::IntegrationElsewhere { expected: a(), actual: b() },
    "integration is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, neither origin nor aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, the commit the landing state says was landed; nothing was moved");
case!(rit_land_is_prefixed_with_publish, PublishError::Land(LandError::NoZone { commit: a() }),
    "publish: squash commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries no Mortar-Zone trailer, so its landing cannot name the zone; nothing was moved");
case!(rit_not_a_landing_names_the_commit_its_parents_and_what_a_landing_has, CulpritError::NotALanding { commit: a(), parents: 3 },
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not a landing: it has 3 parents, and a landing has one (a fast-forward) or two (a merge)");
case!(rit_not_same_change_names_both_commits_and_ids, RereviewError::NotSameChange { reviewed: a(), reviewed_id: Some(id()), new: b(), new_id: None },
    "not the same change: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries Mortar-Change-Id kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb carries none");
case!(rit_not_same_change_without_reviewed_id_prints_none, RereviewError::NotSameChange { reviewed: a(), reviewed_id: None, new: b(), new_id: Some(id()) },
    "not the same change: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries Mortar-Change-Id none, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb carries kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
case!(rit_no_parent_names_the_commit, RereviewError::NoParent { commit: a() },
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa has no parent to compare against");
case!(rit_attr_source_missing_names_the_commit_and_says_nothing_moved, SquashError::AttrSourceMissing { commit: a() },
    "attribute source commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa does not exist, so the merge cannot read its attributes; no commit was written and no ref was moved");
case!(rit_unrelated_names_both_tips_and_says_nothing_moved, SquashError::Unrelated { origin: a(), source: b() },
    "zone tip bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb shares no history with origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; nothing was squashed and no ref was moved");
