//! decide_withdraw: every branch with literal ids: no state, a state naming another commit, the state naming this
//! commit (Landing), the commit in origin (Published), and Published winning when both hold.

use super::{decide_withdraw, WithdrawError};
use crate::landing::LandingState;
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
        change_id: ChangeId::parse(CHANGE).unwrap(),
        zone: "land".to_string(),
        squash: oid(squash),
        origin_at_start: oid(ORIGIN),
        integration: oid(squash),
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
