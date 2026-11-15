//! replayed_position: without a record the source is mapped at the first, middle and last position; a record is
//! followed instead of the source, also out of the range; a commit not in the range and empty ranges are NotOwed;
//! a commit in the range with a result one commit short, or an old or new range that is not one chain, is
//! Unmappable.

use super::{replayed_position, RecordMapping};
use gitio::Oid;

const O1: &str = "1111111111111111111111111111111111111111";
const O2: &str = "2222222222222222222222222222222222222222";
const O3: &str = "3333333333333333333333333333333333333333";
const N1: &str = "4444444444444444444444444444444444444444";
const N2: &str = "5555555555555555555555555555555555555555";
const N3: &str = "6666666666666666666666666666666666666666";
const ELSEWHERE: &str = "7777777777777777777777777777777777777777";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn oids(hexes: &[&str]) -> Vec<Oid> {
    hexes.iter().map(|hex| oid(hex)).collect()
}

/// Both ranges one chain each: old O1, O2, O3 replayed to N1, N2, N3; `source` with no record.
fn chained(source: &str) -> RecordMapping {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2, N3]));
    replayed_position(&oid(source), None, &old, &old, &new, &new)
}

#[test]
fn without_a_record_the_source_is_found_at_the_first_middle_and_last_position() {
    assert_eq!(chained(O1), RecordMapping::Mapped(oid(N1)));
    assert_eq!(chained(O2), RecordMapping::Mapped(oid(N2)));
    assert_eq!(chained(O3), RecordMapping::Mapped(oid(N3)));
}

#[test]
fn a_record_is_followed_instead_of_the_source() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2, N3]));
    assert_eq!(replayed_position(&oid(O1), Some(&oid(O3)), &old, &old, &new, &new), RecordMapping::Mapped(oid(N3)));
}

#[test]
fn a_record_outside_the_range_is_not_owed_even_when_the_source_is_in_it() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2, N3]));
    assert_eq!(replayed_position(&oid(O1), Some(&oid(ELSEWHERE)), &old, &old, &new, &new), RecordMapping::NotOwed);
}

#[test]
fn a_commit_not_in_the_range_is_not_owed() {
    assert_eq!(chained(ELSEWHERE), RecordMapping::NotOwed);
}

#[test]
fn a_result_one_commit_short_is_unmappable() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2]));
    assert_eq!(replayed_position(&oid(O1), None, &old, &old, &new, &new), RecordMapping::Unmappable);
}

#[test]
fn an_old_range_that_is_not_one_chain_is_unmappable() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2, N3]));
    assert_eq!(replayed_position(&oid(O3), None, &old, &oids(&[O1, O3]), &new, &new), RecordMapping::Unmappable);
}

#[test]
fn a_new_range_that_is_not_one_chain_is_unmappable() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2, N3]));
    assert_eq!(replayed_position(&oid(O3), None, &old, &old, &new, &oids(&[N1, N3])), RecordMapping::Unmappable);
}

#[test]
fn a_short_result_or_a_broken_chain_without_the_commit_in_the_range_is_not_owed() {
    let (old, new) = (oids(&[O1, O2, O3]), oids(&[N1, N2]));
    assert_eq!(replayed_position(&oid(ELSEWHERE), None, &old, &oids(&[O1, O3]), &new, &new), RecordMapping::NotOwed);
}

#[test]
fn empty_ranges_are_not_owed() {
    assert_eq!(replayed_position(&oid(O1), None, &[], &[], &[], &[]), RecordMapping::NotOwed);
}
