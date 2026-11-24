//! LandingState::members: a batch of one yields its newest member; a batch of three yields earlier then newest, in
//! landing order.

use super::{LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

/// The one helper building values only a constructor makes (Oid::parse, ChangeId::parse).
fn member(zone: &str, id: &str, hex: char) -> LandingMember {
    LandingMember {
        change_id: ChangeId::parse(id).unwrap(),
        zone: zone.to_string(),
        squash: Oid::parse(&hex.to_string().repeat(40)).unwrap(),
        integration: Oid::parse(&hex.to_string().repeat(40)).unwrap(),
    }
}

#[test]
fn members_are_earlier_then_newest_in_landing_order() {
    let a = member("gitio", "klmnopqrstuvwxyzklmnopqrstuvwxyz", '2');
    let b = member("oplog", "zyxwvutsrqponmlkzyxwvutsrqponmlk", '3');
    let c = member("land", "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", '4');
    let origin = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let rows = [
        ("batch of one", vec![], c.clone(), vec!["land"]),
        ("batch of three", vec![a, b], c, vec!["gitio", "oplog", "land"]),
    ];
    for (name, earlier, newest, expected) in rows {
        let state = LandingState { origin_at_start: origin.clone(), earlier, newest, pending: None };
        let zones: Vec<&str> = state.members().map(|member| member.zone.as_str()).collect();
        assert_eq!(zones, expected, "{name}");
    }
}
