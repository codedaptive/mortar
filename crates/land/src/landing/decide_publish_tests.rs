//! decide_publish: push-on and push-off decisions, including open and done push intents and a batch of two, using
//! literal ids.

use super::{decide_publish, LandingMember, LandingState, PublishStep, PushIntentState};
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
        origin_at_start: oid(EXPECTED),
        earlier: Vec::new(),
        newest: LandingMember {
            change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
            zone: "land".to_string(),
            squash: oid("4444444444444444444444444444444444444444"),
            integration: oid(NEW),
        },
        pending: None,
    }
}

#[test]
fn push_off_decisions_ignore_the_remote() {
    let landing = state();
    let expected = oid(EXPECTED);
    let new = oid(NEW);
    let other = oid(OTHER);
    let rows = [
        ("no state", None, &expected, None, PublishStep::Nothing),
        ("missing remote, expected local", None, &expected, Some(&landing), PublishStep::Advance),
        ("missing remote, new local", None, &new, Some(&landing), PublishStep::Cleanup),
        ("missing remote, other local", None, &other, Some(&landing), PublishStep::BackOut { onto: oid(OTHER) }),
        ("expected remote, expected local", Some(&expected), &expected, Some(&landing), PublishStep::Advance),
        ("expected remote, new local", Some(&expected), &new, Some(&landing), PublishStep::Cleanup),
        ("expected remote, other local", Some(&expected), &other, Some(&landing), PublishStep::BackOut { onto: oid(OTHER) }),
        ("new remote, expected local", Some(&new), &expected, Some(&landing), PublishStep::Advance),
        ("new remote, new local", Some(&new), &new, Some(&landing), PublishStep::Cleanup),
        ("new remote, other local", Some(&new), &other, Some(&landing), PublishStep::BackOut { onto: oid(OTHER) }),
        ("other remote, expected local", Some(&other), &expected, Some(&landing), PublishStep::Advance),
        ("other remote, new local", Some(&other), &new, Some(&landing), PublishStep::Cleanup),
        ("other remote, other local", Some(&other), &other, Some(&landing), PublishStep::BackOut { onto: oid(OTHER) }),
    ];
    let failures: Vec<_> = rows.into_iter().filter_map(|(name, remote, local, state, answer)| {
        let actual = decide_publish(false, &PushIntentState::Absent, remote, local, state);
        (actual != answer).then(|| format!("{name}: expected {answer:?}, got {actual:?}"))
    }).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn push_intent_decisions_finish_the_started_run() {
    let landing = state();
    let expected = oid(EXPECTED);
    let new = oid(NEW);
    let other = oid(OTHER);
    let open = PushIntentState::Open { op_id: "op-1".to_string() };
    let push_open = PublishStep::PushOpen { op_id: "op-1".to_string() };
    let rows = [
        ("no state, push off, open", false, &open, None, &expected, None, PublishStep::Nothing),
        ("no state, push on, open", true, &open, Some(&expected), &new, None, PublishStep::Nothing),
        ("open, off, expected, no remote", false, &open, None, &expected, Some(&landing), push_open.clone()),
        ("open, off, new, no remote", false, &open, None, &new, Some(&landing), push_open.clone()),
        ("open, off, other, no remote", false, &open, None, &other, Some(&landing), push_open.clone()),
        ("open, off, expected, expected remote", false, &open, Some(&expected), &expected, Some(&landing), push_open.clone()),
        ("open, off, new, expected remote", false, &open, Some(&expected), &new, Some(&landing), push_open.clone()),
        ("open, off, other, expected remote", false, &open, Some(&expected), &other, Some(&landing), push_open),
        ("done, off, expected", false, &PushIntentState::Done, None, &expected, Some(&landing), PublishStep::Advance),
        ("done, off, new", false, &PushIntentState::Done, None, &new, Some(&landing), PublishStep::Cleanup),
        ("open, on, new", true, &open, None, &new, Some(&landing), PublishStep::Cleanup),
        ("open, on, expected", true, &open, Some(&expected), &expected, Some(&landing), PublishStep::Push),
        ("done, on, new", true, &PushIntentState::Done, None, &new, Some(&landing), PublishStep::Cleanup),
        ("done, on, expected", true, &PushIntentState::Done, Some(&expected), &expected, Some(&landing), PublishStep::Push),
    ];
    let failures: Vec<_> = rows.into_iter().filter_map(|(name, push, intent, remote, local, state, answer)| {
        let actual = decide_publish(push, intent, remote, local, state);
        (actual != answer).then(|| format!("{name}: expected {answer:?}, got {actual:?}"))
    }).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn no_state_is_nothing_whatever_the_tips() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, Some(&oid(EXPECTED)), &oid(EXPECTED), None), PublishStep::Nothing);
    assert_eq!(decide_publish(true, &PushIntentState::Absent, None, &oid(NEW), None), PublishStep::Nothing);
}

#[test]
fn local_at_new_is_cleanup_without_the_remote() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, None, &oid(NEW), Some(&state())), PublishStep::Cleanup);
}

#[test]
fn local_at_expected_and_remote_at_new_is_advance() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, Some(&oid(NEW)), &oid(EXPECTED), Some(&state())), PublishStep::Advance);
}

#[test]
fn local_and_remote_at_expected_is_push() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, Some(&oid(EXPECTED)), &oid(EXPECTED), Some(&state())), PublishStep::Push);
}

#[test]
fn local_at_expected_and_remote_elsewhere_is_fetch_origin() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, Some(&oid(OTHER)), &oid(EXPECTED), Some(&state())), PublishStep::FetchOrigin);
}

#[test]
fn local_at_expected_and_no_remote_branch_is_remote_missing() {
    assert_eq!(decide_publish(true, &PushIntentState::Absent, None, &oid(EXPECTED), Some(&state())), PublishStep::RemoteMissing);
}

#[test]
fn local_elsewhere_backs_out_onto_the_local_tip() {
    assert_eq!(
        decide_publish(true, &PushIntentState::Absent, None, &oid(OTHER), Some(&state())),
        PublishStep::BackOut { onto: oid("3333333333333333333333333333333333333333") }
    );
}

/// A batch of two: the earlier member landed at FIRST, the newest at NEW, so new is the newest member's integration.
fn batch_of_two() -> LandingState {
    let earlier = LandingMember {
        change_id: ChangeId::parse("zyxwvutsrqponmlkzyxwvutsrqponmlk").unwrap(),
        zone: "other".to_string(),
        squash: oid("5555555555555555555555555555555555555555"),
        integration: oid("6666666666666666666666666666666666666666"),
    };
    LandingState { earlier: vec![earlier], ..state() }
}

#[test]
fn batch_of_two_publishes_from_origin_at_start_to_the_newest_members_integration() {
    let batch = batch_of_two();
    let expected = oid(EXPECTED);
    let new = oid(NEW);
    let rows = [
        ("local at new", None, &new, PublishStep::Cleanup),
        ("local and remote at expected", Some(&expected), &expected, PublishStep::Push),
        ("local at expected, remote at new", Some(&new), &expected, PublishStep::Advance),
    ];
    let failures: Vec<_> = rows.into_iter().filter_map(|(name, remote, local, answer)| {
        let actual = decide_publish(true, &PushIntentState::Absent, remote, local, Some(&batch));
        (actual != answer).then(|| format!("{name}: expected {answer:?}, got {actual:?}"))
    }).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
