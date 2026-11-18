//! Joins open_remote_intent -> read_marked_log -> remote_intent_verdict (find_done_in, open_intents_in) on a
//! log seeded with encode_marked_record lines: a done key is Repeated, an open marked intent with the key is
//! AlreadyOpen, both appending nothing; failed records, unmarked intents, closed intents and other keys
//! leave the verdict empty, so a new marked intent is Opened.

use crate::support::Fixture;
use oplog::{
    encode_marked_record, open_remote_intent, oplog_path, read_marked_log, MarkedRecord, Op, OpPhase, OpRecord,
    OpResult, RemoteIntentOutcome,
};
use std::fs;

fn marked(op_id: &str, key: &str, phase: OpPhase, remote: Option<&str>) -> MarkedRecord {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    MarkedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: key.to_string(),
            kind: "publish".to_string(),
            actor: "simms".to_string(),
            reason: "push to origin".to_string(),
            at: 1763030400,
            phase,
            moves: Vec::new(),
            result,
        },
        values: Vec::new(),
        remote: remote.map(str::to_string),
    }
}

/// The op every test opens: key "k", no moves, so only the verdict decides.
fn op_k() -> Op {
    Op {
        op_id: "op-new".to_string(),
        key: "k".to_string(),
        kind: "publish".to_string(),
        actor: "simms".to_string(),
        reason: "push to origin".to_string(),
        at: 1763030400,
        moves: Vec::new(),
    }
}

/// A fixture whose log holds `log`, one encode_marked_record line each (no log file for an empty `log`): the
/// bytes append_marked_record appends, written directly because no export appends an arbitrary marked record.
fn seeded(log: &[MarkedRecord]) -> Fixture {
    let f = Fixture::new();
    if !log.is_empty() {
        let path = oplog_path(&f.common);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text: String =
            log.iter().map(|m| encode_marked_record(&m.record, &m.values, m.remote.as_deref()) + "\n").collect();
        fs::write(&path, text).unwrap();
    }
    f
}

/// open_remote_intent for op_k on "origin"; the outcome must leave the log's bytes as they were.
fn verdict_writing_nothing(f: &Fixture) -> RemoteIntentOutcome {
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let outcome = open_remote_intent(&f.git, &f.common, &op_k(), "origin").unwrap();
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    outcome
}

/// No verdict: open_remote_intent opens op_k's intent marked "origin", appended after the seeded `log`.
fn assert_opened(f: &Fixture, log: &[MarkedRecord]) {
    let expected = marked("op-new", "k", OpPhase::Intent, Some("origin"));
    let outcome = open_remote_intent(&f.git, &f.common, &op_k(), "origin").unwrap();
    assert_eq!(outcome, RemoteIntentOutcome::Opened(expected.clone()));
    let read = read_marked_log(&f.common).unwrap();
    assert!(read.torn_lines.is_empty());
    let mut all = log.to_vec();
    all.push(expected);
    assert_eq!(read.records, all);
}

#[test]
fn an_empty_log_opens_the_intent() {
    let f = seeded(&[]);
    assert_opened(&f, &[]);
}

#[test]
fn a_done_record_with_the_key_is_repeated() {
    let f = seeded(&[
        marked("op-1", "k", OpPhase::Intent, Some("origin")),
        marked("op-1", "k", OpPhase::Done, Some("origin")),
    ]);
    assert_eq!(
        verdict_writing_nothing(&f),
        RemoteIntentOutcome::Repeated(marked("op-1", "k", OpPhase::Done, Some("origin")))
    );
}

#[test]
fn done_wins_over_an_open_marked_intent_with_the_key() {
    let f = seeded(&[
        marked("op-1", "k", OpPhase::Done, None),
        marked("op-2", "k", OpPhase::Intent, Some("origin")),
    ]);
    assert_eq!(
        verdict_writing_nothing(&f),
        RemoteIntentOutcome::Repeated(marked("op-1", "k", OpPhase::Done, None))
    );
}

#[test]
fn an_open_marked_intent_with_the_key_is_already_open() {
    let f = seeded(&[marked("op-1", "k", OpPhase::Intent, Some("origin"))]);
    assert_eq!(
        verdict_writing_nothing(&f),
        RemoteIntentOutcome::AlreadyOpen(marked("op-1", "k", OpPhase::Intent, Some("origin")))
    );
}

#[test]
fn the_last_of_two_open_marked_intents_is_already_open() {
    let f = seeded(&[
        marked("op-1", "k", OpPhase::Intent, Some("origin")),
        marked("op-2", "k", OpPhase::Intent, Some("backup")),
    ]);
    assert_eq!(
        verdict_writing_nothing(&f),
        RemoteIntentOutcome::AlreadyOpen(marked("op-2", "k", OpPhase::Intent, Some("backup")))
    );
}

#[test]
fn a_marked_intent_closed_failed_later_opens_the_intent() {
    let log = [
        marked("op-1", "k", OpPhase::Intent, Some("origin")),
        marked("op-1", "k", OpPhase::Failed, Some("origin")),
    ];
    assert_opened(&seeded(&log), &log);
}

#[test]
fn a_failed_record_alone_opens_the_intent() {
    let log = [marked("op-1", "k", OpPhase::Failed, None)];
    assert_opened(&seeded(&log), &log);
}

#[test]
fn an_open_unmarked_intent_with_the_key_opens_the_intent() {
    let log = [marked("op-1", "k", OpPhase::Intent, None)];
    assert_opened(&seeded(&log), &log);
}

#[test]
fn records_with_another_key_open_the_intent() {
    let log = [
        marked("op-1", "other", OpPhase::Done, None),
        marked("op-2", "k-longer", OpPhase::Intent, Some("origin")),
    ];
    assert_opened(&seeded(&log), &log);
}
