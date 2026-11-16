//! history_matches: Ref by any move's exact refname, Key and Actor by exact field.

use gitio::Oid;

use super::{history_matches, HistoryFilter};
use crate::store::{OpPhase, OpRecord, RefMove};

fn rec(key: &str, actor: &str, refs: &[&str]) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: key.to_string(),
        kind: "land".to_string(),
        actor: actor.to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase: OpPhase::Intent,
        moves: refs
            .iter()
            .map(|r| RefMove {
                refname: r.to_string(),
                old: None,
                new: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
            })
            .collect(),
        result: None,
    }
}

fn by_ref(refname: &str) -> HistoryFilter {
    HistoryFilter::Ref(refname.to_string())
}

#[test]
fn ref_matches_the_second_of_two_moves() {
    let r = rec("land:zone-b", "nagatha", &["refs/heads/main", "refs/heads/landing"]);
    assert!(history_matches(&r, &by_ref("refs/heads/landing")));
}

#[test]
fn ref_does_not_match_a_record_with_no_moves() {
    let r = rec("lock:land", "simms", &[]);
    assert!(!history_matches(&r, &by_ref("refs/heads/landing")));
}

#[test]
fn ref_does_not_match_a_refname_it_is_only_a_prefix_of() {
    let r = rec("land:zone-a", "simms", &["refs/heads/landing"]);
    assert!(!history_matches(&r, &by_ref("refs/heads/land")));
}

#[test]
fn key_matches_exactly() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(history_matches(&r, &HistoryFilter::Key("land:zone-a".to_string())));
}

#[test]
fn key_does_not_match_a_different_key() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(!history_matches(&r, &HistoryFilter::Key("land:zone-b".to_string())));
}

#[test]
fn key_does_not_match_a_prefix_of_the_key() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(!history_matches(&r, &HistoryFilter::Key("land:zone".to_string())));
}

#[test]
fn key_does_not_match_a_different_case() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(!history_matches(&r, &HistoryFilter::Key("LAND:ZONE-A".to_string())));
}

#[test]
fn actor_matches_exactly() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(history_matches(&r, &HistoryFilter::Actor("simms".to_string())));
}

#[test]
fn actor_does_not_match_a_prefix_of_the_actor() {
    let r = rec("land:zone-a", "simms", &[]);
    assert!(!history_matches(&r, &HistoryFilter::Actor("sim".to_string())));
}
