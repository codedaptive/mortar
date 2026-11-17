//! bisect_report: a culprit (exit 0) naming its landing, its trailers and the next step in its zone; no culprit
//! (exit 1) for each reason; every probe as a line and in the "probes" field, in the order it ran.

use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use land::{BisectAnswer, BisectNotFound, BisectReport, Culprit, Probe, ProbeOutcome};
use oplog::ChangeId;

use super::bisect_report;
use crate::report::{ExitCode, OutValue, Report};

const LAST: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const MID: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90";
const CULPRIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const FAILED: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const CHANGE: &str = "zqxnmwkrtpvsloyuytlmnwqxrkopzsvu";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn probe(hex: &str, outcome: ProbeOutcome) -> Probe {
    Probe { commit: oid(hex), outcome, folder: PathBuf::from("/tmp/mortar-probe") }
}

fn two_probes() -> Vec<Probe> {
    vec![probe(MID, ProbeOutcome::Passed), probe(CULPRIT, ProbeOutcome::Failed { code: Some(1) })]
}

fn culprit(change_id: Option<&str>, zone: Option<&str>) -> BisectReport {
    BisectReport {
        answer: BisectAnswer::Culprit(Culprit {
            commit: oid(CULPRIT),
            change_id: change_id.map(|id| ChangeId::parse(id).unwrap()),
            zone: zone.map(ToString::to_string),
        }),
        probes: two_probes(),
    }
}

fn not_found(reason: BisectNotFound) -> BisectReport {
    BisectReport { answer: BisectAnswer::NotFound(reason), probes: two_probes() }
}

fn report(answer: &BisectReport) -> Report {
    bisect_report(answer, &oid(LAST), &oid(FAILED))
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn probes_field(lines: &[&str]) -> (String, OutValue) {
    ("probes".to_string(), OutValue::Texts(lines.iter().map(ToString::to_string).collect()))
}

const MID_PASSED: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: passed";
const CULPRIT_FAILED: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1";

#[test]
fn a_culprit_with_a_zone_names_the_landing_and_sends_the_fix_to_that_zone() {
    assert_eq!(
        report(&culprit(Some(CHANGE), Some("cli"))),
        Report {
            verb: "bisect".to_string(),
            exit: ExitCode::Done,
            outcome: "culprit".to_string(),
            lines: vec![
                "the first failing landing is 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e".to_string(),
                "its squash is change zqxnmwkrtpvsloyuytlmnwqxrkopzsvu from zone cli".to_string(),
                "probe 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: passed".to_string(),
                "probe 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1".to_string(),
            ],
            owner: None,
            next: "fix the break in zone cli through a request (mortar folder request cli <request> <path> --paths \
                   <path>... --actor <name>), then mortar squash cli --actor <name>"
                .to_string(),
            fields: vec![
                text("commit", CULPRIT),
                text("change_id", CHANGE),
                text("zone", "cli"),
                probes_field(&[MID_PASSED, CULPRIT_FAILED]),
            ],
        }
    );
}

#[test]
fn a_culprit_without_trailers_has_null_fields_and_a_next_step_for_the_owning_zone() {
    assert_eq!(
        report(&culprit(None, None)),
        Report {
            verb: "bisect".to_string(),
            exit: ExitCode::Done,
            outcome: "culprit".to_string(),
            lines: vec![
                "the first failing landing is 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e".to_string(),
                "it carries no squash trailers, so no change ID or zone is named".to_string(),
                "probe 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: passed".to_string(),
                "probe 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1".to_string(),
            ],
            owner: None,
            next: "fix the break in the zone that owns it through a request (mortar folder request <zone> <request> \
                   <path> --paths <path>... --actor <name>), then mortar squash <zone> --actor <name>"
                .to_string(),
            fields: vec![
                text("commit", CULPRIT),
                ("change_id".to_string(), OutValue::Null),
                ("zone".to_string(), OutValue::Null),
                probes_field(&[MID_PASSED, CULPRIT_FAILED]),
            ],
        }
    );
}

#[test]
fn a_culprit_with_a_change_id_and_no_zone_names_the_id_and_leaves_the_zone_null() {
    let answer = report(&culprit(Some(CHANGE), None));
    assert_eq!(answer.lines[1], "its squash is change zqxnmwkrtpvsloyuytlmnwqxrkopzsvu; it carries no Mortar-Zone trailer");
    assert_eq!(
        answer.next,
        "fix the break in the zone that owns it through a request (mortar folder request <zone> <request> <path> \
         --paths <path>... --actor <name>), then mortar squash <zone> --actor <name>"
    );
    assert_eq!(answer.fields[1], text("change_id", CHANGE));
    assert_eq!(answer.fields[2], ("zone".to_string(), OutValue::Null));
}

#[test]
fn a_culprit_with_a_zone_and_no_change_id_still_sends_the_fix_to_that_zone() {
    let answer = report(&culprit(None, Some("join")));
    assert_eq!(answer.lines[1], "its squash is from zone join; it carries no Mortar-Change-Id trailer");
    assert_eq!(
        answer.next,
        "fix the break in zone join through a request (mortar folder request join <request> <path> --paths \
         <path>... --actor <name>), then mortar squash join --actor <name>"
    );
    assert_eq!(answer.fields[1], ("change_id".to_string(), OutValue::Null));
    assert_eq!(answer.fields[2], text("zone", "join"));
}

#[test]
fn the_failed_commit_passing_is_not_found_and_asks_to_check_the_command() {
    assert_eq!(
        report(&not_found(BisectNotFound::FailedPasses)),
        Report {
            verb: "bisect".to_string(),
            exit: ExitCode::Refused,
            outcome: "not-found".to_string(),
            lines: vec![
                "no culprit: the failed commit e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 passes the test command"
                    .to_string(),
                "probe 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: passed".to_string(),
                "probe 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1".to_string(),
            ],
            owner: Some("the test command".to_string()),
            next: "check that the test command fails at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391, then repeat the \
                   same command"
                .to_string(),
            fields: vec![
                text("last_pass", LAST),
                text("failed", FAILED),
                probes_field(&[MID_PASSED, CULPRIT_FAILED]),
            ],
        }
    );
}

#[test]
fn the_last_pass_failing_is_not_found_and_asks_for_an_earlier_passing_commit() {
    assert_eq!(
        report(&not_found(BisectNotFound::LastPassFails)),
        Report {
            verb: "bisect".to_string(),
            exit: ExitCode::Refused,
            outcome: "not-found".to_string(),
            lines: vec![
                "no culprit: the last passing commit 4b825dc642cb6eb9a060e54bf8d69288fbee4904 fails the test \
                 command, so the break is at or before it"
                    .to_string(),
                "probe 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: passed".to_string(),
                "probe 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1".to_string(),
            ],
            owner: Some("the test command".to_string()),
            next: "repeat with an earlier passing commit: mortar bisect <last pass> \
                   e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 --timeout <seconds> --actor <name> -- <command>..."
                .to_string(),
            fields: vec![
                text("last_pass", LAST),
                text("failed", FAILED),
                probes_field(&[MID_PASSED, CULPRIT_FAILED]),
            ],
        }
    );
}

#[test]
fn every_probe_outcome_is_a_line_and_a_field_entry_in_the_order_it_ran() {
    let answer = BisectReport {
        answer: BisectAnswer::NotFound(BisectNotFound::FailedPasses),
        probes: vec![
            probe(FAILED, ProbeOutcome::Passed),
            probe(CULPRIT, ProbeOutcome::TimedOut { limit: Duration::from_secs(120) }),
            probe(MID, ProbeOutcome::Failed { code: None }),
            probe(LAST, ProbeOutcome::Failed { code: Some(2) }),
        ],
    };
    let expected = [
        "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: passed",
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: timed out after 120 s, counted as a fail",
        "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: failed, killed by a signal",
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904: failed with exit 2",
    ];
    let result = report(&answer);
    assert_eq!(
        result.lines[1..],
        [
            "probe e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: passed".to_string(),
            "probe 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: timed out after 120 s, counted as a fail".to_string(),
            "probe 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90: failed, killed by a signal".to_string(),
            "probe 4b825dc642cb6eb9a060e54bf8d69288fbee4904: failed with exit 2".to_string(),
        ]
    );
    assert_eq!(result.fields[2], probes_field(&expected));
}

#[test]
fn no_probes_gives_no_probe_lines_and_an_empty_probes_field() {
    let answer = BisectReport { answer: BisectAnswer::NotFound(BisectNotFound::LastPassFails), probes: Vec::new() };
    let result = report(&answer);
    assert_eq!(result.lines.len(), 1);
    assert_eq!(result.fields[2], ("probes".to_string(), OutValue::Texts(Vec::new())));
}
