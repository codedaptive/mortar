//! decide_back_out: every branch with literal ids: Nothing (with and without a state), NoState, Revert and
//! Elsewhere, for a batch of one and for batches of two and three (the tip, the point before the newest).

use super::{decide_back_out, BackOutStep, LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const LANDED: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state() -> LandingState {
    LandingState {
        origin_at_start: oid(ORIGIN),
        earlier: Vec::new(),
        newest: LandingMember {
            change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
            zone: "land".to_string(),
            squash: oid("4444444444444444444444444444444444444444"),
            integration: oid(LANDED),
        },
        pending: None,
    }
}

#[test]
fn integration_at_origin_is_nothing() {
    assert_eq!(decide_back_out(&oid(ORIGIN), &oid(ORIGIN), None), BackOutStep::Nothing);
    assert_eq!(decide_back_out(&oid(ORIGIN), &oid(ORIGIN), Some(&state())), BackOutStep::Nothing);
}

#[test]
fn integration_off_origin_without_a_state_is_no_state() {
    assert_eq!(decide_back_out(&oid(ORIGIN), &oid(LANDED), None), BackOutStep::NoState);
}

#[test]
fn integration_at_the_states_commit_reverts_that_state() {
    assert_eq!(decide_back_out(&oid(ORIGIN), &oid(LANDED), Some(&state())), BackOutStep::Revert(state()));
}

const FIRST: &str = "5555555555555555555555555555555555555555";
const SECOND: &str = "6666666666666666666666666666666666666666";

fn member(change_id: &str, integration: &str) -> LandingMember {
    LandingMember {
        change_id: ChangeId::parse(change_id).unwrap(),
        zone: "land".to_string(),
        squash: oid("4444444444444444444444444444444444444444"),
        integration: oid(integration),
    }
}

/// A batch whose earlier members landed at `earlier`, in order, and whose newest member landed at LANDED.
fn batch(earlier: &[&str]) -> LandingState {
    LandingState {
        earlier: earlier.iter().map(|at| member("zyxwvutsrqponmlkzyxwvutsrqponmlk", at)).collect(),
        ..state()
    }
}

#[test]
fn batch_back_out_reverts_from_the_tip_or_the_point_before_the_newest() {
    let tip = || BackOutStep::Elsewhere { expected: oid("2222222222222222222222222222222222222222") };
    let rows = [
        ("batch of two at its tip", LANDED, batch(&[FIRST]), BackOutStep::Revert(batch(&[FIRST]))),
        ("batch of two before the newest", FIRST, batch(&[FIRST]), BackOutStep::Revert(batch(&[FIRST]))),
        ("batch of two elsewhere", OTHER, batch(&[FIRST]), tip()),
        ("batch of three before the newest", SECOND, batch(&[FIRST, SECOND]), BackOutStep::Revert(batch(&[FIRST, SECOND]))),
        ("batch of three at its first member", FIRST, batch(&[FIRST, SECOND]), tip()),
        ("batch of one at another commit", FIRST, batch(&[]), tip()),
    ];
    let failures: Vec<_> = rows
        .into_iter()
        .filter_map(|(name, integration, state, answer)| {
            let actual = decide_back_out(&oid(ORIGIN), &oid(integration), Some(&state));
            (actual != answer).then(|| format!("{name}: expected {answer:?}, got {actual:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
