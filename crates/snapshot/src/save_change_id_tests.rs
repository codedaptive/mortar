use crate::save_change_id::save_change_id;
use crate::{SnapshotError, SnapshotTarget};
use gitio::Oid;
use oplog::{ChangeId, MortarTrailers};

const OURS: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const THEIRS: &str = "zzzzyyyyxxxxwwwwvvvvuuuuttttssss";

fn head() -> Oid {
    Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap()
}

fn target() -> SnapshotTarget {
    SnapshotTarget { zone: "snapshot".to_string(), request: "WO-snapshot-3".to_string(), actor: "bilby".to_string() }
}

fn trailers(change_id: Option<&str>, request: Option<&str>, zone: Option<&str>) -> MortarTrailers {
    MortarTrailers {
        change_id: change_id.map(|text| ChangeId::parse(text).unwrap()),
        request: request.map(str::to_string),
        zone: zone.map(str::to_string),
    }
}

#[test]
fn head_of_this_request_and_zone_reuses_its_change_id() {
    let found = save_change_id(&head(), &trailers(Some(OURS), Some("WO-snapshot-3"), Some("snapshot")), &target());
    assert_eq!(found.unwrap(), Some(ChangeId::parse("kkkkllllmmmmnnnnooooppppqqqqrrrr").unwrap()));
}

#[test]
fn head_of_this_request_and_zone_without_change_id_is_change_id_missing() {
    match save_change_id(&head(), &trailers(None, Some("WO-snapshot-3"), Some("snapshot")), &target()) {
        Err(SnapshotError::ChangeIdMissing { head }) => {
            assert_eq!(head, Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap())
        }
        other => panic!("expected ChangeIdMissing, got {other:?}"),
    }
}

#[test]
fn zone_tip_without_trailers_draws_a_new_id() {
    assert_eq!(save_change_id(&head(), &trailers(None, None, None), &target()).unwrap(), None);
}

#[test]
fn same_request_in_another_zone_draws_a_new_id() {
    let found = save_change_id(&head(), &trailers(Some(OURS), Some("WO-snapshot-3"), Some("oplog")), &target());
    assert_eq!(found.unwrap(), None);
}

#[test]
fn another_request_in_the_same_zone_draws_a_new_id() {
    let found = save_change_id(&head(), &trailers(Some(OURS), Some("WO-snapshot-2"), Some("snapshot")), &target());
    assert_eq!(found.unwrap(), None);
}

#[test]
fn another_requests_commit_does_not_lend_its_change_id() {
    let found = save_change_id(&head(), &trailers(Some(THEIRS), Some("WO-oplog-5"), Some("oplog")), &target());
    assert_eq!(found.unwrap(), None);
}

#[test]
fn another_requests_commit_without_change_id_is_not_an_error() {
    let found = save_change_id(&head(), &trailers(None, Some("WO-oplog-5"), Some("oplog")), &target());
    assert_eq!(found.unwrap(), None);
}
