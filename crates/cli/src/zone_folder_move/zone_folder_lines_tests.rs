//! zone_folder_lines: what became of one zone's folder, each ZoneFolderOutcome and FolderRefusal in the cli's words.

use std::path::PathBuf;

use gitio::Oid;

use super::zone_folder_lines;
use crate::zone_folder_move::{FolderRefusal, ZoneFolderOutcome};

const TIP: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const NOT_BROUGHT: &str =
    "the zone folder /work/zones/cli was not brought to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

fn lines(outcome: &ZoneFolderOutcome) -> Vec<String> {
    zone_folder_lines("cli", &Oid::parse(TIP).unwrap(), outcome)
}

fn folder() -> PathBuf {
    PathBuf::from("/work/zones/cli")
}

fn refused(why: FolderRefusal) -> ZoneFolderOutcome {
    ZoneFolderOutcome::Refused { folder: folder(), why }
}

#[test]
fn no_folder_says_the_zone_has_none() {
    assert_eq!(lines(&ZoneFolderOutcome::NoFolder), vec!["zone cli has no zone folder".to_string()]);
}

#[test]
fn a_folder_at_the_tip_says_so() {
    assert_eq!(
        lines(&ZoneFolderOutcome::AtTip { folder: folder() }),
        vec!["the zone folder /work/zones/cli is already at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".to_string()]
    );
}

#[test]
fn a_moved_folder_says_it_was_brought_to_the_tip() {
    assert_eq!(
        lines(&ZoneFolderOutcome::Moved { folder: folder() }),
        vec!["brought the zone folder /work/zones/cli to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".to_string()]
    );
}

#[test]
fn paths_in_the_way_are_listed_and_nothing_changed() {
    let why = FolderRefusal::InTheWay(vec![PathBuf::from("src/a.rs"), PathBuf::from("docs/notes.txt")]);
    assert_eq!(
        lines(&refused(why)),
        vec![
            format!("{NOT_BROUGHT}: local changes are in the way at:"),
            "src/a.rs".to_string(),
            "docs/notes.txt".to_string(),
            "nothing in it changed".to_string(),
        ]
    );
}

#[test]
fn unmerged_paths_are_listed_and_nothing_changed() {
    let why = FolderRefusal::Unmerged(vec![PathBuf::from("Cargo.lock")]);
    assert_eq!(
        lines(&refused(why)),
        vec![
            format!("{NOT_BROUGHT}: its index has unmerged entries at:"),
            "Cargo.lock".to_string(),
            "nothing in it changed".to_string(),
        ]
    );
}

#[test]
fn an_index_matching_no_zone_commit_names_the_zone_and_nothing_changed() {
    assert_eq!(
        lines(&refused(FolderRefusal::NoZoneCommit)),
        vec![
            "the zone folder /work/zones/cli was not brought to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: its index \
             matches no commit zone cli has held"
                .to_string(),
            "nothing in it changed".to_string(),
        ]
    );
}

#[test]
fn a_failed_folder_gives_its_why_on_one_line() {
    let outcome =
        ZoneFolderOutcome::Failed { folder: folder(), why: "git could not read its index".to_string() };
    assert_eq!(
        lines(&outcome),
        vec![
            "the zone folder /work/zones/cli was not brought to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: git could \
             not read its index"
                .to_string()
        ]
    );
}
