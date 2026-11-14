//! same_change: Ok with the id only when both commits carry the same Mortar-Change-Id; different ids, either id
//! missing and both missing are each NotSameChange carrying both commits and both ids as found.

use gitio::Oid;
use oplog::{ChangeId, MortarTrailers};

use super::{same_change, RereviewError};

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const ID_A: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";
const ID_B: &str = "zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk";

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

fn id(text: &str) -> ChangeId {
    ChangeId::parse(text).unwrap()
}

fn trailers(change_id: Option<&str>) -> MortarTrailers {
    MortarTrailers { change_id: change_id.map(id), request: None, zone: None }
}

/// Asserts `same_change` refused with exactly these commits and ids.
fn assert_refused(reviewed_id: Option<&str>, new_id: Option<&str>) {
    match same_change(&oid(REVIEWED), &trailers(reviewed_id), &oid(NEW), &trailers(new_id)) {
        Err(RereviewError::NotSameChange { reviewed, reviewed_id: got_reviewed_id, new, new_id: got_new_id }) => {
            assert_eq!(reviewed, oid(REVIEWED));
            assert_eq!(got_reviewed_id, reviewed_id.map(id));
            assert_eq!(new, oid(NEW));
            assert_eq!(got_new_id, new_id.map(id));
        }
        other => panic!("{reviewed_id:?} against {new_id:?} gave {other:?}, not NotSameChange"),
    }
}

#[test]
fn equal_ids_give_that_id() {
    let got = same_change(&oid(REVIEWED), &trailers(Some(ID_A)), &oid(NEW), &trailers(Some(ID_A))).unwrap();
    assert_eq!(got.as_str(), "kzlymxnwovpuqtrssrtqupvowxnymlzk");
}

#[test]
fn different_ids_are_refused() {
    assert_refused(Some(ID_A), Some(ID_B));
}

#[test]
fn reviewed_missing_its_id_is_refused() {
    assert_refused(None, Some(ID_A));
}

#[test]
fn new_missing_its_id_is_refused() {
    assert_refused(Some(ID_A), None);
}

#[test]
fn both_missing_their_ids_are_refused() {
    assert_refused(None, None);
}
