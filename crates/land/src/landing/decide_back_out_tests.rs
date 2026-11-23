//! decide_back_out: every branch with literal ids: Nothing (with and without a state), NoState, Revert and
//! Elsewhere.

use super::{decide_back_out, BackOutStep, LandingState};
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
        change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
        zone: "land".to_string(),
        squash: oid("4444444444444444444444444444444444444444"),
        origin_at_start: oid(ORIGIN),
        integration: oid(LANDED),
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

#[test]
fn integration_elsewhere_names_the_states_commit() {
    assert_eq!(
        decide_back_out(&oid(ORIGIN), &oid(OTHER), Some(&state())),
        BackOutStep::Elsewhere { expected: oid("2222222222222222222222222222222222222222") }
    );
}
