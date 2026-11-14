use crate::decide_snapshot::decide_snapshot;
use crate::previous_snapshot::PreviousSnapshot;
use crate::snapshot_decision::SnapshotDecision;
use gitio::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn no_previous_snapshot_writes() {
    assert_eq!(decide_snapshot(&oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), None), SnapshotDecision::Write);
}

#[test]
fn same_tree_as_previous_is_unchanged_with_its_commit() {
    let previous = PreviousSnapshot {
        commit: oid("cccccccccccccccccccccccccccccccccccccccc"),
        tree: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    };
    assert_eq!(
        decide_snapshot(&oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), Some(&previous)),
        SnapshotDecision::Unchanged(oid("cccccccccccccccccccccccccccccccccccccccc"))
    );
}

#[test]
fn different_tree_from_previous_writes() {
    let previous = PreviousSnapshot {
        commit: oid("cccccccccccccccccccccccccccccccccccccccc"),
        tree: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    };
    assert_eq!(
        decide_snapshot(&oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), Some(&previous)),
        SnapshotDecision::Write
    );
}
