//! decide_landing: one row per decision with literal ids: the repeat of the newest member and of an earlier one,
//! Resume of a first and of a second member, Append, Fresh with no state, with another squash's stale state, with
//! a state from an older origin and with a published batch's state at origin, Busy off the tip of a batch of two,
//! and NoState.

use super::{decide_landing, LandingMember, LandingState, LandingStep};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const SQUASH_A: &str = "2222222222222222222222222222222222222222";
const LANDED_A: &str = "3333333333333333333333333333333333333333";
const SQUASH_B: &str = "4444444444444444444444444444444444444444";
const LANDED_B: &str = "5555555555555555555555555555555555555555";
const OTHER: &str = "6666666666666666666666666666666666666666";
const SQUASH_C: &str = "7777777777777777777777777777777777777777";
const ID_A: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
const ID_B: &str = "zyxwvutsrqponmlkzyxwvutsrqponmlk";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// A batch started at `origin_at_start` with these (change ID, squash, integration) members in landing order.
fn batch(origin_at_start: &str, members: &[(&str, &str, &str)]) -> Option<LandingState> {
    let mut members: Vec<LandingMember> = members
        .iter()
        .map(|(id, squash, integration)| LandingMember {
            change_id: ChangeId::parse(id).unwrap(),
            zone: "land".to_string(),
            squash: oid(squash),
            integration: oid(integration),
        })
        .collect();
    let newest = members.pop().unwrap();
    Some(LandingState { origin_at_start: oid(origin_at_start), earlier: members, newest, pending: None })
}

#[test]
fn decide_landing_answers_each_step_in_order() {
    let two = [(ID_A, SQUASH_A, LANDED_A), (ID_B, SQUASH_B, LANDED_B)];
    let rows = [
        ("repeat of the newest", LANDED_B, batch(ORIGIN, &two), SQUASH_B, LandingStep::Landed(oid(LANDED_B))),
        ("repeat of an earlier member", LANDED_B, batch(ORIGIN, &two), SQUASH_A, LandingStep::Landed(oid(LANDED_A))),
        (
            "resume of a first member",
            ORIGIN,
            batch(ORIGIN, &[(ID_A, SQUASH_A, LANDED_A)]),
            SQUASH_A,
            LandingStep::Resume { from: oid(ORIGIN), to: oid(LANDED_A) },
        ),
        (
            "resume of a second member",
            LANDED_A,
            batch(ORIGIN, &two),
            SQUASH_B,
            LandingStep::Resume { from: oid(LANDED_A), to: oid(LANDED_B) },
        ),
        (
            "append",
            LANDED_A,
            batch(ORIGIN, &[(ID_A, SQUASH_A, LANDED_A)]),
            SQUASH_B,
            LandingStep::Append { onto: oid(LANDED_A) },
        ),
        ("fresh without a state", ORIGIN, None, SQUASH_A, LandingStep::Fresh),
        (
            "fresh past another squash's stale state",
            ORIGIN,
            batch(ORIGIN, &[(ID_A, SQUASH_A, LANDED_A)]),
            SQUASH_B,
            LandingStep::Fresh,
        ),
        (
            "fresh past a state from an older origin",
            ORIGIN,
            batch(OTHER, &[(ID_B, SQUASH_B, LANDED_B)]),
            SQUASH_B,
            LandingStep::Fresh,
        ),
        (
            "fresh past a published batch's state at origin",
            ORIGIN,
            batch(OTHER, &[(ID_A, SQUASH_A, ORIGIN)]),
            SQUASH_B,
            LandingStep::Fresh,
        ),
        (
            "busy off the tip of a batch of two",
            OTHER,
            batch(ORIGIN, &two),
            SQUASH_C,
            LandingStep::Busy(ChangeId::parse("zyxwvutsrqponmlkzyxwvutsrqponmlk").unwrap()),
        ),
        ("no state off origin", OTHER, None, SQUASH_B, LandingStep::NoState),
    ];
    for (name, integration, state, squash, expected) in rows {
        assert_eq!(decide_landing(&oid(ORIGIN), &oid(integration), state.as_ref(), &oid(squash)), expected, "{name}");
    }
}
