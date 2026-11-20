use gitio::Oid;
use snapshot::{SaveOutcome, SavedWithSnapshot};

use super::snapshot_moved;

fn saved(snapshot_written: bool, save_written: bool) -> SavedWithSnapshot {
    SavedWithSnapshot {
        outcome: SaveOutcome {
            commit: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
            tree: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
            written: save_written,
            change_id: None,
            refused: vec![],
        },
        snapshot_written,
    }
}

#[test]
fn saves_own_snapshot_move_counts_even_when_save_is_unchanged() {
    assert!(snapshot_moved(false, &saved(true, false)));
}

#[test]
fn neither_snapshot_move_stays_false_even_when_save_wrote_a_commit() {
    assert!(!snapshot_moved(false, &saved(false, true)));
}

#[test]
fn earlier_snapshot_move_counts_when_saves_snapshot_did_not_move() {
    assert!(snapshot_moved(true, &saved(false, false)));
}
