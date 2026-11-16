//! restore_report: what a done, repeated and failed restore say, with the folders moved, already in place, left
//! alone because their ref has moved on since the record's op, and the zone folders left where they were (S-113,
//! S-115(1)).

use std::path::PathBuf;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult, RefMove};

use super::{restore_report, FolderOnRef};
use crate::report::{ExitCode, OutValue, Report};

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn record(phase: OpPhase, moves: Vec<RefMove>, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-new".to_string(),
        key: "restore-1".to_string(),
        kind: "restore".to_string(),
        actor: "harness-1".to_string(),
        reason: "mortar restore op-1".to_string(),
        at: 7,
        phase,
        moves,
        result,
    }
}

fn two_moves() -> Vec<RefMove> {
    vec![
        RefMove { refname: "refs/heads/req/core/a".to_string(), old: Some(oid('b')), new: oid('c') },
        RefMove { refname: "refs/mortar/snap/core/a".to_string(), old: None, new: oid('0') },
    ]
}

fn zone_folder() -> FolderOnRef {
    FolderOnRef {
        path: PathBuf::from("/w/core"),
        refname: "refs/heads/zone/core".to_string(),
        record: FolderRecord {
            kind: FolderKind::Zone,
            zone: "core".to_string(),
            request: None,
            base: oid('a'),
            work_order_paths: Vec::new(),
            needed_paths: Vec::new(),
            parent: None,
        },
    }
}

fn texts(items: &[&str]) -> OutValue {
    OutValue::Texts(items.iter().map(ToString::to_string).collect())
}

const MOVE_A: &str =
    "refs/heads/req/core/a bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb -> cccccccccccccccccccccccccccccccccccccccc";
const MOVE_SNAP: &str = "refs/mortar/snap/core/a absent -> deleted";

#[test]
fn a_done_restore_lists_moves_folders_and_zone_folders() {
    let outcome = ApplyOutcome::Done(record(OpPhase::Done, two_moves(), Some(OpResult::Moved)));
    let report = restore_report(
        &outcome,
        "op-1",
        &[PathBuf::from("/w/a")],
        &[PathBuf::from("/w/b")],
        &[PathBuf::from("/w/c")],
        &[zone_folder()],
    );
    assert_eq!(
        report,
        Report {
            verb: "restore".to_string(),
            exit: ExitCode::Done,
            outcome: "restored".to_string(),
            lines: vec![
                MOVE_A.to_string(),
                MOVE_SNAP.to_string(),
                "moved the folder /w/a to its ref's restored value".to_string(),
                "the folder /w/b already stands at its ref's restored value".to_string(),
                "the folder /w/c stays: its ref has moved on since op op-new".to_string(),
                "the zone folder /w/core stands on refs/heads/zone/core; its files stay where they were".to_string(),
            ],
            owner: None,
            next: "mortar log --key restore-1".to_string(),
            fields: vec![
                ("op_id".to_string(), OutValue::Text("op-1".to_string())),
                ("key".to_string(), OutValue::Text("restore-1".to_string())),
                ("moves".to_string(), texts(&[MOVE_A, MOVE_SNAP])),
                ("folders_moved".to_string(), texts(&["/w/a"])),
                ("folders_left".to_string(), texts(&["/w/c"])),
                ("zone_folders".to_string(), texts(&["/w/core"])),
            ],
        }
    );
}

#[test]
fn a_repeat_is_already_done_with_the_records_moves() {
    let outcome = ApplyOutcome::Repeated(record(OpPhase::Done, two_moves(), Some(OpResult::Moved)));
    let report = restore_report(&outcome, "op-1", &[], &[], &[], &[]);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "already-done");
    assert_eq!(report.lines, vec![MOVE_A.to_string(), MOVE_SNAP.to_string()]);
    assert_eq!(report.next, "mortar log --key restore-1");
}

#[test]
fn a_repeat_names_each_folder_left_alone_because_its_ref_moved_on() {
    let outcome = ApplyOutcome::Repeated(record(OpPhase::Done, two_moves(), Some(OpResult::Moved)));
    let report = restore_report(&outcome, "op-1", &[], &[], &[PathBuf::from("/w/a")], &[]);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "already-done");
    assert_eq!(
        report.lines,
        vec![
            MOVE_A.to_string(),
            MOVE_SNAP.to_string(),
            "the folder /w/a stays: its ref has moved on since op op-new".to_string(),
        ]
    );
    assert_eq!(report.fields[4], ("folders_left".to_string(), texts(&["/w/a"])));
}

#[test]
fn a_restore_with_no_moves_says_every_ref_already_holds_its_value() {
    let outcome = ApplyOutcome::Done(record(OpPhase::Done, Vec::new(), Some(OpResult::Moved)));
    let report = restore_report(&outcome, "op-1", &[], &[], &[], &[]);
    assert_eq!(report.lines, vec!["every ref already holds its value as of op op-1".to_string()]);
    assert_eq!(report.fields[2], ("moves".to_string(), texts(&[])));
}

#[test]
fn a_failed_restore_names_the_stale_ref_and_points_at_its_log() {
    let stale = OpResult::Stale { refname: "refs/mortar/snap/core/a".to_string(), actual: Some(oid('d')) };
    let outcome = ApplyOutcome::Failed(record(OpPhase::Failed, two_moves(), Some(stale)));
    let report = restore_report(&outcome, "op-1", &[], &[], &[], &[zone_folder()]);
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "not-moved");
    assert_eq!(
        report.lines,
        vec![
            "refs/mortar/snap/core/a was not where it was read (it holds dddddddddddddddddddddddddddddddddddddddd), \
             so nothing moved"
                .to_string(),
            "the zone folder /w/core stands on refs/heads/zone/core; its files stay where they were".to_string(),
        ]
    );
    assert_eq!(report.next, "mortar log --ref refs/mortar/snap/core/a");
}

#[test]
fn a_failed_restore_not_moved_names_its_first_ref() {
    let not_moved =
        OpResult::NotMoved { refs: vec![("refs/heads/req/core/a".to_string(), None), ("refs/x".to_string(), None)] };
    let outcome = ApplyOutcome::Failed(record(OpPhase::Failed, two_moves(), Some(not_moved)));
    let report = restore_report(&outcome, "op-1", &[], &[], &[], &[]);
    assert_eq!(
        report.lines,
        vec!["refs/heads/req/core/a was not where it was read (it holds nothing), so nothing moved"]
    );
    assert_eq!(report.next, "mortar log --ref refs/heads/req/core/a");
}
