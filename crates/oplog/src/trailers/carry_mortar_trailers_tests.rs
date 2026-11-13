//! carry_mortar_trailers: each field is old's when old has it, else new's.

use super::{carry_mortar_trailers, MortarTrailers};
use crate::change_id::ChangeId;

const OLD_ID: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";
const NEW_ID: &str = "zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk";

fn trailers(change_id: Option<&str>, request: Option<&str>, zone: Option<&str>) -> MortarTrailers {
    MortarTrailers {
        change_id: change_id.map(|text| ChangeId::parse(text).unwrap()),
        request: request.map(str::to_string),
        zone: zone.map(str::to_string),
    }
}

#[test]
fn old_full_and_new_full_gives_old() {
    let old = trailers(Some(OLD_ID), Some("WO-old"), Some("oplog"));
    let new = trailers(Some(NEW_ID), Some("WO-new"), Some("join"));
    assert_eq!(carry_mortar_trailers(&old, &new), trailers(Some(OLD_ID), Some("WO-old"), Some("oplog")));
}

#[test]
fn old_empty_gives_new() {
    let new = trailers(Some(NEW_ID), Some("WO-new"), Some("join"));
    let carried = carry_mortar_trailers(&MortarTrailers::default(), &new);
    assert_eq!(carried, trailers(Some(NEW_ID), Some("WO-new"), Some("join")));
}

#[test]
fn both_empty_gives_empty() {
    let carried = carry_mortar_trailers(&MortarTrailers::default(), &MortarTrailers::default());
    assert_eq!(carried, MortarTrailers::default());
}

#[test]
fn mixed_fields_each_take_old_when_present() {
    let old = trailers(Some(OLD_ID), None, Some("oplog"));
    let new = trailers(None, Some("WO-new"), Some("join"));
    assert_eq!(carry_mortar_trailers(&old, &new), trailers(Some(OLD_ID), Some("WO-new"), Some("oplog")));
}

#[test]
fn mixed_fields_each_take_new_when_old_absent() {
    let old = trailers(None, Some("WO-old"), None);
    let new = trailers(Some(NEW_ID), None, Some("join"));
    assert_eq!(carry_mortar_trailers(&old, &new), trailers(Some(NEW_ID), Some("WO-old"), Some("join")));
}

#[test]
fn old_change_id_wins_over_a_different_new_one() {
    let old = trailers(Some(OLD_ID), None, None);
    let new = trailers(Some(NEW_ID), None, None);
    let carried = carry_mortar_trailers(&old, &new);
    assert_eq!(carried.change_id.as_ref().map(ChangeId::as_str), Some(OLD_ID));
}
