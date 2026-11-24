//! resolve_landing_state: no pending, a pending commit at integration's tip (folded into the newest member) and one
//! not at it (dropped), with literal ids; the earlier member of a batch of two is left as it is.

use super::{resolve_landing_state, LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const LANDED: &str = "2222222222222222222222222222222222222222";
const PENDING: &str = "3333333333333333333333333333333333333333";
const FIRST: &str = "5555555555555555555555555555555555555555";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn member(zone: &str, integration: &str) -> LandingMember {
    LandingMember {
        change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
        zone: zone.to_string(),
        squash: oid("4444444444444444444444444444444444444444"),
        integration: oid(integration),
    }
}

fn state(earlier: &[&str], integration: &str, pending: Option<&str>) -> LandingState {
    LandingState {
        origin_at_start: oid(ORIGIN),
        earlier: earlier.iter().map(|at| member("gitio", at)).collect(),
        newest: member("land", integration),
        pending: pending.map(oid),
    }
}

#[test]
fn resolve_landing_state_folds_a_pending_commit_at_the_tip_and_drops_any_other() {
    let rows = [
        ("no pending", state(&[], LANDED, None), LANDED, state(&[], LANDED, None)),
        ("pending at the tip is folded in", state(&[], LANDED, Some(PENDING)), PENDING, state(&[], PENDING, None)),
        ("pending not at the tip is dropped", state(&[], LANDED, Some(PENDING)), LANDED, state(&[], LANDED, None)),
        (
            "pending at the tip of a batch of two goes on the newest",
            state(&[FIRST], LANDED, Some(PENDING)),
            PENDING,
            state(&[FIRST], PENDING, None),
        ),
    ];
    for (name, given, tip, expected) in rows {
        assert_eq!(resolve_landing_state(given, &oid(tip)), expected, "{name}");
    }
}
