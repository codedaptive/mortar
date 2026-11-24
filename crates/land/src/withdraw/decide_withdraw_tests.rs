//! decide_withdraw: every branch with literal ids: no state, a state naming another commit, the state naming this
//! commit (Landing), the commit in origin (Published), Published winning when both hold, and a batch of two whose
//! newest or earlier member is this commit (Landing with that member's change ID) or neither (Ok).

use super::{decide_withdraw, WithdrawError};
use crate::landing::{LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

const COMMIT: &str = "1111111111111111111111111111111111111111";
const ORIGIN: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";
const CHANGE: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(squash: &str) -> LandingState {
    LandingState {
        origin_at_start: oid(ORIGIN),
        earlier: Vec::new(),
        newest: LandingMember {
            change_id: ChangeId::parse(CHANGE).unwrap(),
            zone: "land".to_string(),
            squash: oid(squash),
            integration: oid(squash),
        },
        pending: None,
    }
}

/// What a decision came to, comparable: WithdrawError is not Eq.
#[derive(Debug, PartialEq)]
enum Seen {
    Ok,
    Published { zone: String, commit: Oid, origin: Oid },
    Landing { zone: String, commit: Oid, change_id: ChangeId },
    Other(String),
}

fn seen(decision: Result<(), WithdrawError>) -> Seen {
    match decision {
        Ok(()) => Seen::Ok,
        Err(WithdrawError::Published { zone, commit, origin }) => Seen::Published { zone, commit, origin },
        Err(WithdrawError::Landing { zone, commit, change_id }) => Seen::Landing { zone, commit, change_id },
        Err(other) => Seen::Other(format!("{other:?}")),
    }
}

#[test]
fn decide_withdraw_refuses_a_published_squash_and_the_landing_in_flight() {
    let published = || Seen::Published { zone: "land".to_string(), commit: oid(COMMIT), origin: oid(ORIGIN) };
    let rows = [
        ("no state", false, None, Seen::Ok),
        ("in origin", true, None, published()),
        (
            "state names this commit",
            false,
            Some(state(COMMIT)),
            Seen::Landing {
                zone: "land".to_string(),
                commit: oid(COMMIT),
                change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
            },
        ),
        ("state names another commit", false, Some(state(OTHER)), Seen::Ok),
        ("in origin and state names it", true, Some(state(COMMIT)), published()),
    ];
    for (name, in_origin, state, expected) in rows {
        let decision = decide_withdraw("land", &oid(COMMIT), &oid(ORIGIN), in_origin, state.as_ref());
        assert_eq!(seen(decision), expected, "{name}");
    }
}

const EARLIER_CHANGE: &str = "zyxwvutsrqponmlkzyxwvutsrqponmlk";

/// A batch of two: an earlier member with squash `earlier` and change EARLIER_CHANGE, the newest with squash `newest`
/// and change CHANGE.
fn batch(earlier: &str, newest: &str) -> LandingState {
    let member = LandingMember {
        change_id: ChangeId::parse(EARLIER_CHANGE).unwrap(),
        zone: "other".to_string(),
        squash: oid(earlier),
        integration: oid(earlier),
    };
    LandingState { earlier: vec![member], ..state(newest) }
}

#[test]
fn decide_withdraw_refuses_any_member_of_the_batch() {
    let landing = |change: &str| Seen::Landing {
        zone: "land".to_string(),
        commit: oid("1111111111111111111111111111111111111111"),
        change_id: ChangeId::parse(change).unwrap(),
    };
    let rows = [
        ("newest member", batch(OTHER, COMMIT), landing("klmnopqrstuvwxyzklmnopqrstuvwxyz")),
        ("earlier member", batch(COMMIT, OTHER), landing("zyxwvutsrqponmlkzyxwvutsrqponmlk")),
        ("other squashes only", batch(OTHER, ORIGIN), Seen::Ok),
    ];
    for (name, state, expected) in rows {
        let decision = decide_withdraw("land", &oid(COMMIT), &oid(ORIGIN), false, Some(&state));
        assert_eq!(seen(decision), expected, "{name}");
    }
}
