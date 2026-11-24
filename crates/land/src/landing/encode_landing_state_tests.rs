//! encode_landing_state: the one-landing form for a batch of one (with and without pending, and with a zone needing
//! escapes) and the batch form for a batch of two, each against text written out by hand.

use super::{encode_landing_state, LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn member(change_id: &str, zone: &str, squash: &str, integration: &str) -> LandingMember {
    LandingMember {
        change_id: ChangeId::parse(change_id).unwrap(),
        zone: zone.to_string(),
        squash: oid(squash),
        integration: oid(integration),
    }
}

fn gitio() -> LandingMember {
    member(
        "zyxwvutsrqponmlkzyxwvutsrqponmlk",
        "gitio",
        "4444444444444444444444444444444444444444",
        "5555555555555555555555555555555555555555",
    )
}

fn zone(name: &str) -> LandingMember {
    member(
        "klmnopqrstuvwxyzklmnopqrstuvwxyz",
        name,
        "2222222222222222222222222222222222222222",
        "3333333333333333333333333333333333333333",
    )
}

fn state(earlier: Vec<LandingMember>, newest: LandingMember, pending: Option<&str>) -> LandingState {
    LandingState {
        origin_at_start: oid("1111111111111111111111111111111111111111"),
        earlier,
        newest,
        pending: pending.map(oid),
    }
}

#[test]
fn encode_landing_state_writes_one_landing_alone_and_the_batch_form_from_two() {
    let rows = [
        (
            "batch of one",
            state(vec![], zone("land"), None),
            "{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"integration\":\"3333333333333333333333333333333333333333\"}\n",
        ),
        (
            "batch of two",
            state(vec![gitio()], zone("land"), None),
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"zyxwvutsrqponmlkzyxwvutsrqponmlk\",\"zone\":\"gitio\",\"squash\":\"4444444444444444444444444444444444444444\",\"integration\":\"5555555555555555555555555555555555555555\"},{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}]}\n",
        ),
        (
            "with pending",
            state(vec![], zone("land"), Some("6666666666666666666666666666666666666666")),
            "{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"integration\":\"3333333333333333333333333333333333333333\",\"pending\":\"6666666666666666666666666666666666666666\"}\n",
        ),
        (
            "a zone needing escapes",
            state(vec![], zone("a\"b\\c"), None),
            "{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"a\\\"b\\\\c\",\"squash\":\"2222222222222222222222222222222222222222\",\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"integration\":\"3333333333333333333333333333333333333333\"}\n",
        ),
    ];
    for (name, given, expected) in rows {
        assert_eq!(encode_landing_state(&given), expected, "{name}");
    }
}
