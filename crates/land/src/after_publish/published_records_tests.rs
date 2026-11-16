//! published_records (rule D): the source's own record deleted whatever its value, a behind record deleted, a record
//! not behind kept, records order kept, and each delete's new value the all-zero id of its value's length.

use super::{published_records, ZoneRecord};
use gitio::Oid;
use oplog::RefMove;

const ZONE: &str = "land";
const SOURCE: &str = "1111111111111111111111111111111111111111";
const VALUE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_VALUE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const VALUE_256: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ZERO: &str = "0000000000000000000000000000000000000000";
const ZERO_256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const OWN_REF: &str = "refs/mortar/replayed/land/1111111111111111111111111111111111111111";
const REF_8: &str = "refs/mortar/replayed/land/8888888888888888888888888888888888888888";
const REF_9: &str = "refs/mortar/replayed/land/9999999999999999999999999999999999999999";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(refname: &str, commit: &str, behind: bool) -> ZoneRecord {
    ZoneRecord { refname: refname.to_string(), commit: oid(commit), behind }
}

fn delete(refname: &str, old: &str, zero: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(old)), new: oid(zero) }
}

#[test]
fn the_sources_own_record_is_deleted_even_when_not_behind() {
    let records = [record(OWN_REF, VALUE, false)];
    assert_eq!(published_records(ZONE, &oid(SOURCE), &records).unwrap(), vec![delete(OWN_REF, VALUE, ZERO)]);
}

#[test]
fn a_record_behind_the_held_commit_is_deleted() {
    let records = [record(REF_9, VALUE, true)];
    assert_eq!(published_records(ZONE, &oid(SOURCE), &records).unwrap(), vec![delete(REF_9, VALUE, ZERO)]);
}

#[test]
fn a_record_not_behind_and_not_the_sources_own_is_kept() {
    let records = [record(REF_9, VALUE, false)];
    assert_eq!(published_records(ZONE, &oid(SOURCE), &records).unwrap(), vec![]);
}

#[test]
fn deletes_keep_records_order_and_skip_the_kept_ones() {
    let records = [record(REF_9, VALUE, true), record(REF_8, OTHER_VALUE, false), record(OWN_REF, OTHER_VALUE, false)];
    assert_eq!(
        published_records(ZONE, &oid(SOURCE), &records).unwrap(),
        vec![delete(REF_9, VALUE, ZERO), delete(OWN_REF, OTHER_VALUE, ZERO)]
    );
}

#[test]
fn a_delete_of_a_sha256_value_is_the_64_digit_zero_id() {
    let records = [record(REF_9, VALUE_256, true)];
    assert_eq!(published_records(ZONE, &oid(SOURCE), &records).unwrap(), vec![delete(REF_9, VALUE_256, ZERO_256)]);
}
