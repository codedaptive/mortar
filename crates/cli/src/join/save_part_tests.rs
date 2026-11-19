//! save_part: the finished move, the save and the refused files, as lines and as --json fields in their order.
//! Unit tests: join/finished_move_part is pure and used directly.

use std::path::PathBuf;

use gitio::Oid;
use oplog::ChangeId;
use snapshot::{RefusedFile, SaveOutcome};

use super::{save_part, MoveCause, PendingMove};
use crate::report::OutValue;

const COMMIT: &str = "4444444444444444444444444444444444444444";
const TREE: &str = "5555555555555555555555555555555555555555";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn written(refused: Vec<RefusedFile>) -> SaveOutcome {
    SaveOutcome {
        commit: oid(COMMIT),
        tree: oid(TREE),
        written: true,
        change_id: Some(ChangeId::parse(CHANGE).unwrap()),
        refused,
    }
}

fn unwritten() -> SaveOutcome {
    SaveOutcome { commit: oid(COMMIT), tree: oid(TREE), written: false, change_id: None, refused: Vec::new() }
}

fn field(name: &str, value: OutValue) -> (String, OutValue) {
    (name.to_string(), value)
}

#[test]
fn a_written_save_names_its_commit_and_change() {
    let (lines, fields) = save_part(&written(Vec::new()), None);
    assert_eq!(
        lines,
        vec![
            "work saved at join as 4444444444444444444444444444444444444444 (change kkkkllllmmmmnnnnooooppppqqqqrrrr)"
                .to_string()
        ]
    );
    assert_eq!(
        fields,
        vec![
            field("finished_move_from", OutValue::Null),
            field("finished_move_to", OutValue::Null),
            field("saved_commit", OutValue::Text(COMMIT.to_string())),
            field("saved_written", OutValue::Bool(true)),
            field("change_id", OutValue::Text(CHANGE.to_string())),
            field("refused", OutValue::Texts(Vec::new())),
        ]
    );
}

#[test]
fn an_unwritten_save_says_no_unsaved_work() {
    let (lines, fields) = save_part(&unwritten(), None);
    assert_eq!(lines, vec!["no unsaved work".to_string()]);
    assert_eq!(fields[3], field("saved_written", OutValue::Bool(false)));
    assert_eq!(fields[4], field("change_id", OutValue::Null));
}

#[test]
fn each_refused_file_is_a_line_and_a_path() {
    let refused = vec![
        RefusedFile { path: PathBuf::from("data/big.bin"), size: 6_291_456, cap: 5_242_880 },
        RefusedFile { path: PathBuf::from("out.tar"), size: 9_000_000, cap: 5_242_880 },
    ];
    let (lines, fields) = save_part(&written(refused), None);
    assert_eq!(
        lines[1..],
        [
            "not saved, over the size cap: data/big.bin (6291456 bytes, cap 5242880)".to_string(),
            "not saved, over the size cap: out.tar (9000000 bytes, cap 5242880)".to_string(),
        ]
    );
    assert_eq!(fields[5], field("refused", OutValue::Texts(vec!["data/big.bin".to_string(), "out.tar".to_string()])));
}

#[test]
fn a_finished_move_comes_first() {
    let moved = PendingMove {
        request_ref: "refs/heads/req/core/fix-1".to_string(),
        old: oid(OLD),
        new: oid(NEW),
        op_id: "op-rebase".to_string(),
        folder_tree: None,
        cause: MoveCause::Rebase,
    };
    let (lines, fields) = save_part(&unwritten(), Some(&moved));
    assert_eq!(
        lines,
        vec![
            "the folder stands at the last rebase's tip: 1111111111111111111111111111111111111111 -> \
             2222222222222222222222222222222222222222"
                .to_string(),
            "no unsaved work".to_string(),
        ]
    );
    assert_eq!(fields[0], field("finished_move_from", OutValue::Text(OLD.to_string())));
    assert_eq!(fields[1], field("finished_move_to", OutValue::Text(NEW.to_string())));
}
