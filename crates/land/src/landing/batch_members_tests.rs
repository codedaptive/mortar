//! batch_members: a batch of one (what the one-landing form decodes to) names its one member; a batch of two names
//! the earlier member then the newest.

use super::{batch_members, BatchMember, LandingMember, LandingState};
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

fn named(id: &str, zone: &str) -> BatchMember {
    BatchMember { change_id: ChangeId::parse(id).unwrap(), zone: zone.to_string() }
}

#[test]
fn batch_members_names_every_member_in_landing_order() {
    let gitio = member("gitio", "zyxwvutsrqponmlkzyxwvutsrqponmlk", '2');
    let land = member("land", "klmnopqrstuvwxyzklmnopqrstuvwxyz", '3');
    let rows = [
        ("batch of one", vec![], land.clone(), vec![named("klmnopqrstuvwxyzklmnopqrstuvwxyz", "land")]),
        (
            "batch of two",
            vec![gitio],
            land,
            vec![named("zyxwvutsrqponmlkzyxwvutsrqponmlk", "gitio"), named("klmnopqrstuvwxyzklmnopqrstuvwxyz", "land")],
        ),
    ];
    for (name, earlier, newest, expected) in rows {
        let state = LandingState {
            origin_at_start: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
            earlier,
            newest,
            pending: None,
        };
        assert_eq!(batch_members(&state), expected, "{name}");
    }
}
