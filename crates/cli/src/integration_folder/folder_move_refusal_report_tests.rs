//! folder_move_refusal_report: exit 1 "folder-not-moved" naming the folder, both tips, what blocks the move (the
//! paths in the way, or another reason) and the owner's step in words: clear it, then repeat the same command.

use std::path::{Path, PathBuf};

use gitio::Oid;

use super::{folder_move_refusal_report, MoveBlock};
use crate::report::{ExitCode, OutValue, Report};

/// The words no next may hold: a git command, a by-hand sync, and the claim that
/// integration already moved.
fn assert_next_is_words_only(report: &Report) {
    assert!(!report.next.contains("git "), "a git command in next: {}", report.next);
    assert!(!report.next.contains("bring its files"), "a by-hand sync in next: {}", report.next);
    assert!(!report.next.contains("already moved"), "an already-moved claim in next: {}", report.next);
}

#[test]
fn paths_in_the_way_after_a_back_out_are_listed_with_the_repeat_in_words() {
    let old = Oid::parse("1111111111111111111111111111111111111111").expect("a valid oid");
    let new = Oid::parse("2222222222222222222222222222222222222222").expect("a valid oid");
    let block = MoveBlock::Paths(vec![PathBuf::from("a.txt"), PathBuf::from("dir/b.txt")]);
    let report = folder_move_refusal_report(
        "back-out",
        "integration is back at 2222222222222222222222222222222222222222",
        Path::new("/r/integration"),
        &old,
        &new,
        &block,
        "mr back-out --actor <name>",
    );
    assert_eq!(
        report,
        Report {
            verb: "back-out".to_string(),
            exit: ExitCode::Refused,
            outcome: "folder-not-moved".to_string(),
            lines: vec![
                "integration is back at 2222222222222222222222222222222222222222".to_string(),
                "the integration folder /r/integration could not move from 1111111111111111111111111111111111111111 \
                 to 2222222222222222222222222222222222222222: local changes are in the way at:"
                    .to_string(),
                "a.txt".to_string(),
                "dir/b.txt".to_string(),
                "back-out stopped here".to_string(),
            ],
            owner: Some("the integration folder /r/integration".to_string()),
            next: "move each path listed out of the integration folder /r/integration, or put it back to the \
                   folder's index copy, and leave the index as it is; then repeat the same command: \
                   mr back-out --actor <name>"
                .to_string(),
            fields: vec![
                ("folder".to_string(), OutValue::Text("/r/integration".to_string())),
                ("old".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
                ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
                ("paths".to_string(), OutValue::Texts(vec!["a.txt".to_string(), "dir/b.txt".to_string()])),
            ],
        }
    );
    assert_next_is_words_only(&report);
}

#[test]
fn another_block_at_a_refused_finish_before_land_says_nothing_moved_and_to_repeat() {
    let old = Oid::parse("1111111111111111111111111111111111111111").expect("a valid oid");
    let tip = Oid::parse("2222222222222222222222222222222222222222").expect("a valid oid");
    let block = MoveBlock::Other("its lock could not be taken".to_string());
    let report = folder_move_refusal_report(
        "land",
        "land stopped before its land call and moved nothing: integration is at \
         2222222222222222222222222222222222222222, where an earlier run moved it, and its folder is still at \
         1111111111111111111111111111111111111111",
        Path::new("/r/integration"),
        &old,
        &tip,
        &block,
        "mr land 3333333333333333333333333333333333333333 --actor <name>",
    );
    assert_eq!(
        report,
        Report {
            verb: "land".to_string(),
            exit: ExitCode::Refused,
            outcome: "folder-not-moved".to_string(),
            lines: vec![
                "land stopped before its land call and moved nothing: integration is at \
                 2222222222222222222222222222222222222222, where an earlier run moved it, and its folder is still \
                 at 1111111111111111111111111111111111111111"
                    .to_string(),
                "the integration folder /r/integration could not move from 1111111111111111111111111111111111111111 \
                 to 2222222222222222222222222222222222222222: its lock could not be taken"
                    .to_string(),
                "land stopped here".to_string(),
            ],
            owner: Some("the integration folder /r/integration".to_string()),
            next: "clear what stops the integration folder /r/integration from moving, leaving its index as it is; \
                   then repeat the same command: mr land 3333333333333333333333333333333333333333 --actor <name>"
                .to_string(),
            fields: vec![
                ("folder".to_string(), OutValue::Text("/r/integration".to_string())),
                ("old".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
                ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
            ],
        }
    );
    assert_next_is_words_only(&report);
}
