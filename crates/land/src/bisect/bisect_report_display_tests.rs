//! Report texts: the NotFound answers with no probes, the culprit lines with and without each trailer, and the
//! probe lines indented after the answer in the order run.

use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use oplog::ChangeId;

use super::{BisectAnswer, BisectNotFound, BisectReport, Culprit, Probe, ProbeOutcome};

fn report(answer: BisectAnswer) -> BisectReport {
    BisectReport { answer, probes: Vec::new() }
}

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn id() -> ChangeId { ChangeId::parse("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk").unwrap() }
fn probe(outcome: ProbeOutcome) -> Probe { Probe { commit: a(), outcome, folder: PathBuf::from("unused-probe-folder") } }
fn culprit_report(change_id: Option<ChangeId>, zone: Option<&str>) -> BisectReport {
    BisectReport { answer: BisectAnswer::Culprit(Culprit { commit: a(), change_id, zone: zone.map(str::to_string) }), probes: vec![] }
}

#[test]
fn failed_passes_says_the_failed_commit_passes() {
    assert_eq!(
        report(BisectAnswer::NotFound(BisectNotFound::FailedPasses)).to_string(),
        "no culprit: the failed commit passes the command"
    );
}

#[test]
fn last_pass_fails_says_the_last_passing_commit_fails() {
    assert_eq!(
        report(BisectAnswer::NotFound(BisectNotFound::LastPassFails)).to_string(),
        "no culprit: the last passing commit fails the command"
    );
}

#[test]
fn a_culprit_with_trailers_names_its_commit_change_and_zone() {
    assert_eq!(culprit_report(Some(id()), Some("zone-x")).to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, zone zone-x");
}

#[test]
fn a_culprit_without_trailers_says_it_carries_none() {
    assert_eq!(culprit_report(None, None).to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; it carries no squash trailers");
}

#[test]
fn a_culprit_without_a_zone_names_the_missing_zone_trailer() {
    assert_eq!(culprit_report(Some(id()), None).to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk; it carries no Mortar-Zone trailer");
}

#[test]
fn a_culprit_without_a_change_id_names_the_missing_change_id_trailer() {
    assert_eq!(culprit_report(None, Some("zone-x")).to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, zone zone-x; it carries no Mortar-Change-Id trailer");
}

#[test]
fn probes_follow_the_answer_indented_in_the_order_run() {
    let mut report = culprit_report(Some(id()), Some("zone-x"));
    report.probes = vec![probe(ProbeOutcome::TimedOut { limit: Duration::from_secs(30) }),
        Probe { commit: b(), outcome: ProbeOutcome::Passed, folder: PathBuf::from("unused-second-folder") }];
    assert_eq!(report.to_string(), "culprit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, zone zone-x\n  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 30s\n  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: passed");
}
