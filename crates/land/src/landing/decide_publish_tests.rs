//! decide_publish: every branch with literal ids: Nothing, Cleanup, Advance, Push, FetchOrigin, RemoteMissing and
//! BackOut.

use super::{decide_publish, LandingState, PublishStep};
use gitio::Oid;
use oplog::ChangeId;

const EXPECTED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
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
        origin_at_start: oid(EXPECTED),
        integration: oid(NEW),
    }
}

#[test]
fn no_state_is_nothing_whatever_the_tips() {
    assert_eq!(decide_publish(Some(&oid(EXPECTED)), &oid(EXPECTED), None), PublishStep::Nothing);
    assert_eq!(decide_publish(None, &oid(NEW), None), PublishStep::Nothing);
}

#[test]
fn local_at_new_is_cleanup_without_the_remote() {
    assert_eq!(decide_publish(None, &oid(NEW), Some(&state())), PublishStep::Cleanup);
}

#[test]
fn local_at_expected_and_remote_at_new_is_advance() {
    assert_eq!(decide_publish(Some(&oid(NEW)), &oid(EXPECTED), Some(&state())), PublishStep::Advance);
}

#[test]
fn local_and_remote_at_expected_is_push() {
    assert_eq!(decide_publish(Some(&oid(EXPECTED)), &oid(EXPECTED), Some(&state())), PublishStep::Push);
}

#[test]
fn local_at_expected_and_remote_elsewhere_is_fetch_origin() {
    assert_eq!(decide_publish(Some(&oid(OTHER)), &oid(EXPECTED), Some(&state())), PublishStep::FetchOrigin);
}

#[test]
fn local_at_expected_and_no_remote_branch_is_remote_missing() {
    assert_eq!(decide_publish(None, &oid(EXPECTED), Some(&state())), PublishStep::RemoteMissing);
}

#[test]
fn local_elsewhere_backs_out_onto_the_local_tip() {
    assert_eq!(
        decide_publish(None, &oid(OTHER), Some(&state())),
        PublishStep::BackOut { onto: oid("3333333333333333333333333333333333333333") }
    );
}
