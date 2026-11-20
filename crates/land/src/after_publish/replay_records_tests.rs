//! replay_records: a write per mapped commit (old from an existing record, None when absent), a write whose old
//! equals its new left out, a carry of a record whose value is a mapped source, a write winning over a carry of the
//! same ref, records outside the range left alone, and the order (writes in mapped order, then carries).

use super::{replay_records, ZoneRecord};
use gitio::Oid;
use join::MappedCommit;
use oplog::RefMove;

const ZONE: &str = "land";
const SOURCE_1: &str = "1111111111111111111111111111111111111111";
const SOURCE_2: &str = "2222222222222222222222222222222222222222";
const REPLAYED_1: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REPLAYED_2: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ELSEWHERE: &str = "cccccccccccccccccccccccccccccccccccccccc";
const REF_1: &str = "refs/mortar/replayed/land/1111111111111111111111111111111111111111";
const REF_2: &str = "refs/mortar/replayed/land/2222222222222222222222222222222222222222";
const REF_8: &str = "refs/mortar/replayed/land/8888888888888888888888888888888888888888";
const REF_9: &str = "refs/mortar/replayed/land/9999999999999999999999999999999999999999";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn mapped(source: &str, replayed: &str) -> MappedCommit {
    MappedCommit { source: oid(source), replayed: oid(replayed) }
}

fn record(refname: &str, commit: &str) -> ZoneRecord {
    ZoneRecord { refname: refname.to_string(), commit: oid(commit), behind: false }
}

fn moved(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

#[test]
fn each_mapped_commit_writes_its_record_created_when_absent() {
    let mapped = [mapped(SOURCE_1, REPLAYED_1), mapped(SOURCE_2, REPLAYED_2)];
    assert_eq!(
        replay_records(ZONE, &mapped, &[]),
        vec![moved(REF_1, None, REPLAYED_1), moved(REF_2, None, REPLAYED_2)]
    );
}

#[test]
fn a_write_over_an_existing_record_names_its_value_as_old() {
    let records = [record(REF_1, ELSEWHERE)];
    assert_eq!(
        replay_records(ZONE, &[mapped(SOURCE_1, REPLAYED_1)], &records),
        vec![moved(REF_1, Some(ELSEWHERE), REPLAYED_1)]
    );
}

#[test]
fn a_write_whose_record_already_holds_the_replay_is_left_out() {
    let records = [record(REF_1, REPLAYED_1)];
    assert_eq!(replay_records(ZONE, &[mapped(SOURCE_1, REPLAYED_1)], &records), vec![]);
}

#[test]
fn a_record_whose_value_is_a_mapped_source_is_carried_to_its_replay() {
    let records = [record(REF_9, SOURCE_1)];
    assert_eq!(
        replay_records(ZONE, &[mapped(SOURCE_1, REPLAYED_1)], &records),
        vec![moved(REF_1, None, REPLAYED_1), moved(REF_9, Some(SOURCE_1), REPLAYED_1)]
    );
}

#[test]
fn a_write_wins_over_a_carry_of_the_same_ref() {
    // REF_2's value SOURCE_1 is a mapped source (a carry to REPLAYED_1), but REF_2 is SOURCE_2's own record.
    let records = [record(REF_2, SOURCE_1)];
    let mapped = [mapped(SOURCE_1, REPLAYED_1), mapped(SOURCE_2, REPLAYED_2)];
    assert_eq!(
        replay_records(ZONE, &mapped, &records),
        vec![moved(REF_1, None, REPLAYED_1), moved(REF_2, Some(SOURCE_1), REPLAYED_2)]
    );
}

#[test]
fn records_whose_value_is_not_in_the_range_are_left_alone() {
    let records = [record(REF_9, ELSEWHERE)];
    assert_eq!(replay_records(ZONE, &[mapped(SOURCE_1, REPLAYED_1)], &records), vec![moved(REF_1, None, REPLAYED_1)]);
}

#[test]
fn no_mapped_commit_moves_no_record() {
    let records = [record(REF_9, SOURCE_1), record(REF_1, ELSEWHERE)];
    assert_eq!(replay_records(ZONE, &[], &records), vec![]);
}

#[test]
fn writes_come_in_mapped_order_then_carries_in_records_order() {
    let records = [record(REF_8, SOURCE_2), record(REF_9, SOURCE_1)];
    let mapped = [mapped(SOURCE_2, REPLAYED_2), mapped(SOURCE_1, REPLAYED_1)];
    assert_eq!(
        replay_records(ZONE, &mapped, &records),
        vec![
            moved(REF_2, None, REPLAYED_2),
            moved(REF_1, None, REPLAYED_1),
            moved(REF_8, Some(SOURCE_2), REPLAYED_2),
            moved(REF_9, Some(SOURCE_1), REPLAYED_1),
        ]
    );
}
