//! pick_mortar_trailers: Mortar's three keys picked case-insensitively, other keys ignored, the last of a
//! repeated key winning, and every malformed Mortar-Change-Id refused naming its text.

use super::{pick_mortar_trailers, MortarTrailers};
use crate::change_id::{ChangeId, ChangeIdError};

const ID_A: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";
const ID_B: &str = "zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk";

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn id(text: &str) -> ChangeId {
    ChangeId::parse(text).unwrap()
}

fn assert_refused_naming(list: &[(&str, &str)], bad: &str) {
    match pick_mortar_trailers(&pairs(list)) {
        Err(err @ ChangeIdError::Malformed { .. }) => {
            let message = err.to_string();
            assert!(message.contains(&format!("{bad:?}")), "{message} does not name {bad:?}");
        }
        other => panic!("{list:?} gave {other:?}, not Malformed"),
    }
}

#[test]
fn no_pairs_is_all_absent() {
    assert_eq!(pick_mortar_trailers(&[]).unwrap(), MortarTrailers::default());
}

#[test]
fn only_foreign_keys_is_all_absent() {
    let list = [("Signed-off-by", "A <a@example.com>"), ("Change-Id", "I0123abcd"), ("Mortar", "x")];
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap(), MortarTrailers::default());
}

#[test]
fn all_three_present_fill_each_field() {
    let list = [
        ("Signed-off-by", "A <a@example.com>"),
        ("Mortar-Change-Id", ID_A),
        ("Mortar-Request", "r1"),
        ("Mortar-Zone", "oplog"),
    ];
    let expected = MortarTrailers {
        change_id: Some(id(ID_A)),
        request: Some("r1".to_string()),
        zone: Some("oplog".to_string()),
    };
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap(), expected);
}

#[test]
fn lower_case_keys_match() {
    let list = [("mortar-change-id", ID_A), ("mortar-request", "r1"), ("mortar-zone", "z1")];
    let expected = MortarTrailers {
        change_id: Some(id(ID_A)),
        request: Some("r1".to_string()),
        zone: Some("z1".to_string()),
    };
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap(), expected);
}

#[test]
fn upper_case_keys_match() {
    let list = [("MORTAR-CHANGE-ID", ID_B), ("MORTAR-REQUEST", "r2"), ("MORTAR-ZONE", "z2")];
    let expected = MortarTrailers {
        change_id: Some(id(ID_B)),
        request: Some("r2".to_string()),
        zone: Some("z2".to_string()),
    };
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap(), expected);
}

#[test]
fn repeated_change_id_takes_the_last() {
    let list = [("Mortar-Change-Id", ID_A), ("Mortar-Change-Id", ID_B)];
    let picked = pick_mortar_trailers(&pairs(&list)).unwrap();
    assert_eq!(picked.change_id.as_ref().map(ChangeId::as_str), Some(ID_B));
}

#[test]
fn repeated_request_takes_the_last() {
    let list = [("Mortar-Request", "first"), ("Mortar-Zone", "z"), ("mortar-request", "second")];
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap().request.as_deref(), Some("second"));
}

#[test]
fn repeated_zone_takes_the_last() {
    let list = [("Mortar-Zone", "gitio"), ("Mortar-Zone", "oplog")];
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap().zone.as_deref(), Some("oplog"));
}

#[test]
fn malformed_change_id_alone_is_refused_naming_it() {
    assert_refused_naming(&[("Mortar-Change-Id", "abc123")], "abc123");
}

#[test]
fn malformed_then_valid_change_id_is_refused_naming_the_bad_one() {
    let bad = "kzlymxnwovpuqtrssrtqupvowxnymlz";
    assert_refused_naming(&[("Mortar-Change-Id", bad), ("Mortar-Change-Id", ID_A)], bad);
}

#[test]
fn absent_change_id_with_request_and_zone() {
    let list = [("Mortar-Request", "r2"), ("Mortar-Zone", "join")];
    let expected = MortarTrailers {
        change_id: None,
        request: Some("r2".to_string()),
        zone: Some("join".to_string()),
    };
    assert_eq!(pick_mortar_trailers(&pairs(&list)).unwrap(), expected);
}
