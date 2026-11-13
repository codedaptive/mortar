//! mortar_trailer_pairs: present trailers in the order Mortar-Change-Id, Mortar-Request, Mortar-Zone,
//! absent ones skipped; and pick_mortar_trailers reads back what it gives.

use super::{mortar_trailer_pairs, pick_mortar_trailers, MortarTrailers};
use crate::change_id::ChangeId;

const ID_A: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";

fn full() -> MortarTrailers {
    MortarTrailers {
        change_id: Some(ChangeId::parse(ID_A).unwrap()),
        request: Some("WO-oplog-4".to_string()),
        zone: Some("oplog".to_string()),
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn all_three_in_order() {
    let expected = pairs(&[
        ("Mortar-Change-Id", ID_A),
        ("Mortar-Request", "WO-oplog-4"),
        ("Mortar-Zone", "oplog"),
    ]);
    assert_eq!(mortar_trailer_pairs(&full()), expected);
}

#[test]
fn absent_change_id_is_skipped() {
    let trailers = MortarTrailers { change_id: None, ..full() };
    let expected = pairs(&[("Mortar-Request", "WO-oplog-4"), ("Mortar-Zone", "oplog")]);
    assert_eq!(mortar_trailer_pairs(&trailers), expected);
}

#[test]
fn absent_request_is_skipped() {
    let trailers = MortarTrailers { request: None, ..full() };
    let expected = pairs(&[("Mortar-Change-Id", ID_A), ("Mortar-Zone", "oplog")]);
    assert_eq!(mortar_trailer_pairs(&trailers), expected);
}

#[test]
fn absent_zone_is_skipped() {
    let trailers = MortarTrailers { zone: None, ..full() };
    let expected = pairs(&[("Mortar-Change-Id", ID_A), ("Mortar-Request", "WO-oplog-4")]);
    assert_eq!(mortar_trailer_pairs(&trailers), expected);
}

#[test]
fn all_absent_is_empty() {
    assert_eq!(mortar_trailer_pairs(&MortarTrailers::default()), Vec::new());
}

#[test]
fn pick_reads_back_what_pairs_gives() {
    let ids = [None, Some("kzlymxnwovpuqtrssrtqupvowxnymlzk"), Some("zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk")];
    let texts = [None, Some("WO-a"), Some("with spaces: and a colon")];
    for change_id in ids {
        for request in texts {
            for zone in texts {
                let trailers = MortarTrailers {
                    change_id: change_id.map(|text| ChangeId::parse(text).unwrap()),
                    request: request.map(str::to_string),
                    zone: zone.map(str::to_string),
                };
                let back = pick_mortar_trailers(&mortar_trailer_pairs(&trailers)).unwrap();
                assert_eq!(back, trailers);
            }
        }
    }
}
