//! decode_landing_state: the batch form (a batch of one, a batch of two, with pending, a zone needing escapes), the
//! one-landing form read as a batch of one (with and without pending), and the batch form refused for an empty
//! landings array and a missing key, each from text written out by hand.

use super::{decode_landing_state, LandingMember, LandingState};
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

fn state(earlier: Vec<LandingMember>, newest: LandingMember, pending: Option<&str>) -> Result<LandingState, String> {
    Ok(LandingState {
        origin_at_start: oid("1111111111111111111111111111111111111111"),
        earlier,
        newest,
        pending: pending.map(oid),
    })
}

#[test]
fn decode_landing_state_reads_both_forms_and_refuses_a_batch_without_a_member_or_a_key() {
    let rows = [
        (
            "batch of one",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}]}\n",
            state(vec![], zone("land"), None),
        ),
        (
            "batch of two, the last member the newest",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"zyxwvutsrqponmlkzyxwvutsrqponmlk\",\"zone\":\"gitio\",\"squash\":\"4444444444444444444444444444444444444444\",\"integration\":\"5555555555555555555555555555555555555555\"},{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}]}\n",
            state(vec![gitio()], zone("land"), None),
        ),
        (
            "with pending",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}],\"pending\":\"6666666666666666666666666666666666666666\"}\n",
            state(vec![], zone("land"), Some("6666666666666666666666666666666666666666")),
        ),
        (
            "a zone needing escapes",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"a\\\"b\\\\c\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}]}\n",
            state(vec![], zone("a\"b\\c"), None),
        ),
        (
            "the one-landing form is a batch of one",
            "{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"integration\":\"3333333333333333333333333333333333333333\"}\n",
            state(vec![], zone("land"), None),
        ),
        (
            "the one-landing form with pending",
            "{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"land\",\"squash\":\"2222222222222222222222222222222222222222\",\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"integration\":\"3333333333333333333333333333333333333333\",\"pending\":\"6666666666666666666666666666666666666666\"}\n",
            state(vec![], zone("land"), Some("6666666666666666666666666666666666666666")),
        ),
        (
            "an empty landings array is refused",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[]}\n",
            Err("expected a landing in the landings array at byte 74: it is empty".to_string()),
        ),
        (
            "a member missing its zone is refused",
            "{\"origin_at_start\":\"1111111111111111111111111111111111111111\",\"landings\":[{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"squash\":\"2222222222222222222222222222222222222222\",\"integration\":\"3333333333333333333333333333333333333333\"}]}\n",
            Err("expected ,\"zone\": at byte 121".to_string()),
        ),
    ];
    for (name, text, expected) in rows {
        assert_eq!(decode_landing_state(text), expected, "{name}");
    }
}
