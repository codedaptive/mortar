//! Display for LandError's own refusals, each as literal text saying nothing moved: OpFailed and OpRepeated name the
//! op, NoZone, NoLandingState and IntegrationElsewhere their commits. The State arm prefixes LandingStateError's text
//! with "landing: "; the Squash, Git, Oplog and Trailers arms embed another error's text and have no case here.

use super::{LandError, LandingStateError};
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult};
use std::path::PathBuf;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:refs/heads/integration:a:b:op-7".to_string(),
        kind: "land".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    assert_eq!(
        LandError::OpFailed(record(OpPhase::Failed)).to_string(),
        "op op-7 (key land:refs/heads/integration:a:b:op-7) failed: integration was not at its expected value, so \
         nothing was moved; run again to read the refs afresh"
    );
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    assert_eq!(
        LandError::OpRepeated(record(OpPhase::Done)).to_string(),
        "op key land:refs/heads/integration:a:b:op-7 is already done in the op log, so this attempt moved nothing; \
         run again"
    );
}

#[test]
fn no_zone_names_the_commit_and_says_nothing_moved() {
    assert_eq!(LandError::NoZone { commit: a() }.to_string(), "squash commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries no Mortar-Zone trailer, so its landing cannot name the zone; nothing was moved");
}

#[test]
fn no_landing_state_names_both_tips_and_says_nothing_moved() {
    assert_eq!(LandError::NoLandingState { origin: a(), integration: b() }.to_string(), "integration is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, not at origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, and no landing state names a landing in flight; nothing was moved");
}

#[test]
fn integration_elsewhere_names_both_commits_and_says_nothing_moved() {
    assert_eq!(LandError::IntegrationElsewhere { expected: a(), actual: b() }.to_string(), "integration is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, neither origin nor aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, the commit the landing state says was landed; nothing was moved");
}

#[test]
fn state_is_prefixed_with_landing() {
    let inner = LandingStateError::Corrupt { path: PathBuf::from("/r/mortar/landing.json"), reason: "x".to_string() };
    let text = LandError::State(inner).to_string();
    assert_eq!(text, "landing: landing state /r/mortar/landing.json is not a state land wrote: x");
}
