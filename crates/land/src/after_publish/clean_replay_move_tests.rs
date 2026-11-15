//! clean_replay_move: NotOwed moves the zone with no record, Move moves it with the record, Unmappable leaves it at
//! its tip as ReplayDropped naming the source.

use super::{clean_replay_move, RecordRead, ZoneMove};
use gitio::Oid;
use oplog::RefMove;
use std::ops::ControlFlow;

const TIP: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const SOURCE: &str = "3333333333333333333333333333333333333333";
const RECORDED: &str = "4444444444444444444444444444444444444444";
const RECORD_REF: &str = "refs/mortar/replayed/land/3333333333333333333333333333333333333333";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record() -> RefMove {
    RefMove { refname: RECORD_REF.to_string(), old: Some(oid(RECORDED)), new: oid(NEW) }
}

#[test]
fn no_record_owed_moves_the_zone_with_no_record() {
    assert_eq!(
        clean_replay_move("land", &oid(TIP), oid(NEW), RecordRead::NotOwed),
        ControlFlow::Continue((oid(NEW), None))
    );
}

#[test]
fn a_record_move_moves_the_zone_with_it() {
    assert_eq!(
        clean_replay_move("land", &oid(TIP), oid(NEW), RecordRead::Move(record())),
        ControlFlow::Continue((oid(NEW), Some(record())))
    );
}

#[test]
fn an_unmappable_record_leaves_the_zone_at_its_tip_naming_the_source() {
    assert_eq!(
        clean_replay_move("land", &oid(TIP), oid(NEW), RecordRead::Unmappable { source: oid(SOURCE) }),
        ControlFlow::Break(ZoneMove::ReplayDropped { zone: "land".to_string(), tip: oid(TIP), squash_of: oid(SOURCE) })
    );
}
