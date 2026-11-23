//! resolve_landing_state: no pending, a pending commit at integration's tip (folded in) and one not at it (dropped),
//! with literal ids.

use super::{resolve_landing_state, LandingState};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const LANDED: &str = "2222222222222222222222222222222222222222";
const PENDING: &str = "3333333333333333333333333333333333333333";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(integration: &str, pending: Option<&str>) -> LandingState {
    LandingState {
        change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
        zone: "land".to_string(),
        squash: oid("4444444444444444444444444444444444444444"),
        origin_at_start: oid(ORIGIN),
        integration: oid(integration),
        pending: pending.map(oid),
    }
}

#[test]
fn resolve_landing_state_folds_a_pending_commit_at_the_tip_and_drops_any_other() {
    let rows = [
        ("no pending", state(LANDED, None), LANDED, state(LANDED, None)),
        ("pending at the tip is folded in", state(LANDED, Some(PENDING)), PENDING, state(PENDING, None)),
        ("pending not at the tip is dropped", state(LANDED, Some(PENDING)), LANDED, state(LANDED, None)),
    ];
    for (name, given, tip, expected) in rows {
        assert_eq!(resolve_landing_state(given, &oid(tip)), expected, "{name}");
    }
}
