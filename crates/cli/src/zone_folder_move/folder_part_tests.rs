//! folder_part: each ZoneFolderOutcome and FolderRefusal mapped to the FOLDER_PARTS index it needs, or none.

use std::path::PathBuf;

use super::folder_part;
use crate::zone_folder_move::{FolderRefusal, ZoneFolderOutcome};

#[test]
fn each_outcome_needs_its_folder_part() {
    let folder = || PathBuf::from("/work/zones/cli");
    let refused = |why: FolderRefusal| ZoneFolderOutcome::Refused { folder: folder(), why };
    let rows: Vec<(&str, ZoneFolderOutcome, Option<usize>)> = vec![
        ("no folder", ZoneFolderOutcome::NoFolder, None),
        ("at tip", ZoneFolderOutcome::AtTip { folder: folder() }, None),
        ("moved", ZoneFolderOutcome::Moved { folder: folder() }, None),
        ("in the way", refused(FolderRefusal::InTheWay(vec![PathBuf::from("a.rs")])), Some(0)),
        ("unmerged", refused(FolderRefusal::Unmerged(vec![PathBuf::from("b.rs")])), Some(1)),
        ("no zone commit", refused(FolderRefusal::NoZoneCommit), Some(2)),
        ("failed", ZoneFolderOutcome::Failed { folder: folder(), why: "git failed".to_string() }, Some(3)),
    ];
    for (name, outcome, expected) in rows {
        assert_eq!(folder_part(&outcome), expected, "row {name}");
    }
}
