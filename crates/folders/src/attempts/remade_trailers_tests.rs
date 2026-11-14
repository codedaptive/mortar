//! remade_trailers: the parent's change ID, request and zone when all three are there and name this request and
//! zone; otherwise none, including for trailers that could not be read.

use oplog::{ChangeId, MortarTrailers};

use super::remade_trailers;

const ID: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn trailers(change_id: Option<&str>, request: Option<&str>, zone: Option<&str>) -> MortarTrailers {
    MortarTrailers {
        change_id: change_id.map(|id| ChangeId::parse(id).unwrap()),
        request: request.map(str::to_string),
        zone: zone.map(str::to_string),
    }
}

#[test]
fn a_parent_of_this_request_and_zone_passes_its_three_trailers_on() {
    let parent = trailers(Some(ID), Some("r"), Some("z"));
    assert_eq!(remade_trailers("z", "r", Some(&parent)), trailers(Some(ID), Some("r"), Some("z")));
}

#[test]
fn unreadable_parent_trailers_give_none() {
    assert_eq!(remade_trailers("z", "r", None), MortarTrailers::default());
}

#[test]
fn a_parent_with_no_change_id_gives_none() {
    let parent = trailers(None, Some("r"), Some("z"));
    assert_eq!(remade_trailers("z", "r", Some(&parent)), MortarTrailers::default());
}

#[test]
fn a_parent_of_another_request_gives_none() {
    let parent = trailers(Some(ID), Some("other"), Some("z"));
    assert_eq!(remade_trailers("z", "r", Some(&parent)), MortarTrailers::default());
}

#[test]
fn a_parent_of_another_zone_gives_none() {
    let parent = trailers(Some(ID), Some("r"), Some("other"));
    assert_eq!(remade_trailers("z", "r", Some(&parent)), MortarTrailers::default());
}

#[test]
fn a_parent_missing_its_request_or_zone_gives_none() {
    assert_eq!(remade_trailers("z", "r", Some(&trailers(Some(ID), None, Some("z")))), MortarTrailers::default());
    assert_eq!(remade_trailers("z", "r", Some(&trailers(Some(ID), Some("r"), None))), MortarTrailers::default());
}

#[test]
fn a_parent_with_no_trailers_gives_none() {
    assert_eq!(remade_trailers("z", "r", Some(&MortarTrailers::default())), MortarTrailers::default());
}
