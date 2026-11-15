//! consumed_record: Advance and Replay with a record consume it; without a record, at Current, and for every read
//! or case that runs no move (not publishing, Missing, the refusals) nothing is consumed.

use super::{consumed_record, SquashOfRead, ZoneCase};
use gitio::Oid;

const SOURCE: &str = "1111111111111111111111111111111111111111";
const RECORDED: &str = "2222222222222222222222222222222222222222";
const BASE: &str = "3333333333333333333333333333333333333333";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn found(record: Option<&str>) -> SquashOfRead {
    SquashOfRead::Found { commit: oid(SOURCE), held: Some(oid(RECORDED)), record: record.map(oid) }
}

#[test]
fn an_advance_with_a_record_consumes_it() {
    assert_eq!(consumed_record(&ZoneCase::Advance, &found(Some(RECORDED))), Some(oid(RECORDED)));
}

#[test]
fn a_replay_with_a_record_consumes_it() {
    assert_eq!(consumed_record(&ZoneCase::Replay { base: oid(BASE) }, &found(Some(RECORDED))), Some(oid(RECORDED)));
}

#[test]
fn an_advance_or_a_replay_without_a_record_consumes_nothing() {
    assert_eq!(consumed_record(&ZoneCase::Advance, &found(None)), None);
    assert_eq!(consumed_record(&ZoneCase::Replay { base: oid(BASE) }, &found(None)), None);
}

#[test]
fn current_with_a_record_consumes_nothing() {
    assert_eq!(consumed_record(&ZoneCase::Current, &found(Some(RECORDED))), None);
}

#[test]
fn a_zone_that_did_not_publish_consumes_nothing() {
    assert_eq!(consumed_record(&ZoneCase::Advance, &SquashOfRead::NotPublishing), None);
    assert_eq!(consumed_record(&ZoneCase::Replay { base: oid(BASE) }, &SquashOfRead::NotPublishing), None);
}

#[test]
fn a_missing_squash_of_consumes_nothing() {
    assert_eq!(consumed_record(&ZoneCase::NoSquashOf, &SquashOfRead::Missing), None);
}

#[test]
fn the_refusals_consume_nothing_even_with_a_record() {
    assert_eq!(consumed_record(&ZoneCase::NoSquashOf, &found(Some(RECORDED))), None);
    let refused = ZoneCase::SquashOfNotInZone { squash_of: oid(SOURCE) };
    assert_eq!(consumed_record(&refused, &found(Some(RECORDED))), None);
}
