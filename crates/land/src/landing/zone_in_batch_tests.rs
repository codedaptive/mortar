//! zone_in_batch on a batch of gitio, oplog then land: a zone no member holds, an earlier member's zone and the
//! newest member's zone.

use super::{zone_in_batch, LandingMember, LandingState};
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
fn zone_in_batch_answers_the_member_holding_the_zone() {
    let state = LandingState {
        origin_at_start: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        earlier: vec![
            member("gitio", "klmnopqrstuvwxyzklmnopqrstuvwxyz", '2'),
            member("oplog", "zyxwvutsrqponmlkzyxwvutsrqponmlk", '3'),
        ],
        newest: member("land", "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", '4'),
        pending: None,
    };
    let rows = [
        ("no member holds it", "join", None),
        ("earlier member", "oplog", Some("zyxwvutsrqponmlkzyxwvutsrqponmlk")),
        ("newest member", "land", Some("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk")),
    ];
    for (name, zone, expected) in rows {
        let found = zone_in_batch(&state, zone).map(|member| member.change_id.as_str());
        assert_eq!(found, expected, "{name}");
    }
}
