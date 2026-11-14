//! decide_landing: every branch with literal ids: the repeat, Busy, NoState, Resume, and Fresh with no state, with
//! another squash's stale state and with a state from an older origin.

use super::{decide_landing, LandingState, LandingStep};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const SQUASH: &str = "2222222222222222222222222222222222222222";
const LANDED: &str = "3333333333333333333333333333333333333333";
const OTHER: &str = "4444444444444444444444444444444444444444";
const ID: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(squash: &str, origin_at_start: &str, integration: &str) -> LandingState {
    LandingState {
        change_id: ChangeId::parse(ID).unwrap(),
        zone: "land".to_string(),
        squash: oid(squash),
        origin_at_start: oid(origin_at_start),
        integration: oid(integration),
    }
}

#[test]
fn this_squash_already_at_integration_is_landed() {
    let s = state(SQUASH, ORIGIN, LANDED);
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(LANDED), Some(&s), &oid(SQUASH)), LandingStep::Landed(oid(LANDED)));
}

#[test]
fn a_fast_forward_repeat_is_landed() {
    let s = state(SQUASH, ORIGIN, SQUASH);
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(SQUASH), Some(&s), &oid(SQUASH)), LandingStep::Landed(oid(SQUASH)));
}

#[test]
fn integration_off_origin_with_another_squash_in_flight_is_busy() {
    let s = state(OTHER, ORIGIN, LANDED);
    assert_eq!(
        decide_landing(&oid(ORIGIN), &oid(LANDED), Some(&s), &oid(SQUASH)),
        LandingStep::Busy(ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap())
    );
}

#[test]
fn integration_off_origin_and_not_at_this_states_commit_is_busy() {
    let s = state(SQUASH, ORIGIN, LANDED);
    assert_eq!(
        decide_landing(&oid(ORIGIN), &oid(OTHER), Some(&s), &oid(SQUASH)),
        LandingStep::Busy(ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap())
    );
}

#[test]
fn integration_off_origin_without_a_state_is_no_state() {
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(LANDED), None, &oid(SQUASH)), LandingStep::NoState);
}

#[test]
fn this_squashs_state_from_this_origin_with_integration_unmoved_resumes() {
    let s = state(SQUASH, ORIGIN, LANDED);
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(ORIGIN), Some(&s), &oid(SQUASH)), LandingStep::Resume(oid(LANDED)));
}

#[test]
fn no_state_at_origin_is_fresh() {
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(ORIGIN), None, &oid(SQUASH)), LandingStep::Fresh);
}

#[test]
fn another_squashs_state_at_origin_is_fresh() {
    let s = state(OTHER, ORIGIN, LANDED);
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(ORIGIN), Some(&s), &oid(SQUASH)), LandingStep::Fresh);
}

#[test]
fn this_squashs_state_from_an_older_origin_is_fresh() {
    let s = state(SQUASH, OTHER, LANDED);
    assert_eq!(decide_landing(&oid(ORIGIN), &oid(ORIGIN), Some(&s), &oid(SQUASH)), LandingStep::Fresh);
}
