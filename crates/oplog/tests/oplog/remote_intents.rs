use crate::support::{operation, refs, test_op, Fixture};
use gitio::Oid;
use oplog::{
    append_op_record, apply, close_intent, intent_record, open_intent, open_remote_intent, oplog_path,
    read_marked_log, read_op_log, recover, IntentClose, Op, OpPhase, OpResult, OplogError, RefMove,
    RemoteIntentOutcome,
};
use std::fs;

/// The JSON text of `operation(objects)` written by hand: the members before `at`, and its moves array.
fn operation_text(objects: &[Oid; 3]) -> (String, String) {
    let (old, new) = (&objects[0], &objects[1]);
    let head = r#"{"op_id":"integration-op-1","key":"integration-repeat-key","kind":"join","actor":"integration-actor","reason":"move two refs atomically""#;
    let moves = format!(
        r#"[{{"ref":"refs/mortar/a","old":"{old}","new":"{new}"}},{{"ref":"refs/mortar/b","old":"{old}","new":"{new}"}}]"#
    );
    (head.to_string(), moves)
}

/// An op whose one move is on a branch the remote holds, as publish's push intent is.
pub fn remote_op(id: &str, objects: &[Oid; 3]) -> Op {
    test_op(id, "publish-caller", vec![RefMove {
        refname: "refs/heads/main".into(),
        old: Some(objects[0].clone()),
        new: objects[1].clone(),
    }])
}

fn opened(outcome: RemoteIntentOutcome) -> oplog::MarkedRecord {
    match outcome {
        RemoteIntentOutcome::Opened(m) => m,
        other => panic!("expected Opened: {other:?}"),
    }
}

#[test]
fn open_remote_intent_writes_one_marked_intent_and_moves_no_ref() {
    // Joins open_remote_intent -> check_ref_names -> append_marked_record -> read_marked_log.
    let f = Fixture::new();
    let intent = opened(open_remote_intent(&f.git, &f.common, &operation(&f.objects), "origin").unwrap());
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[0]));
    assert_eq!(intent.remote.as_deref(), Some("origin"));
    assert_eq!(intent.record.phase, OpPhase::Intent);
    assert_eq!(intent.record.result, None);
    assert!(intent.values.is_empty());
    let log = read_marked_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records, [intent]);
    let text = fs::read_to_string(oplog_path(&f.common)).unwrap();
    let (head, moves) = operation_text(&f.objects);
    assert_eq!(text, format!("{head},\"at\":100,\"phase\":\"intent\",\"moves\":{moves},\"remote\":\"origin\"}}\n"));
    assert!(text.ends_with(",\"remote\":\"origin\"}\n"));
}

#[test]
fn repeat_while_open_is_already_open_and_after_done_is_repeated() {
    // Joins open_remote_intent's remote_intent_verdict -> close_intent -> the verdict on the done record.
    let f = Fixture::new();
    let op = operation(&f.objects);
    let intent = opened(open_remote_intent(&f.git, &f.common, &op, "origin").unwrap());
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let again = open_remote_intent(&f.git, &f.common, &op, "origin").unwrap();
    assert_eq!(again, RemoteIntentOutcome::AlreadyOpen(intent));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    let done = close_intent(&f.common, "integration-op-1", IntentClose::Done).unwrap();
    assert_eq!(done.record.phase, OpPhase::Done);
    assert_eq!(done.remote.as_deref(), Some("origin"));
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let repeat = open_remote_intent(&f.git, &f.common, &op, "origin").unwrap();
    assert_eq!(repeat, RemoteIntentOutcome::Repeated(done));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(String::from_utf8(bytes).unwrap().lines().count(), 2);
}

#[test]
fn bad_ref_name_is_bad_move_and_writes_nothing() {
    // Joins open_remote_intent -> check_ref_names (git check-ref-format) before any append.
    let f = Fixture::new();
    let mut bad = remote_op("op-bad", &f.objects);
    bad.moves[0].refname = "refs/heads/a..b".into();
    let refused = |f: &Fixture| match open_remote_intent(&f.git, &f.common, &bad, "origin") {
        Err(OplogError::BadMove { refname, .. }) => refname,
        other => panic!("expected BadMove: {other:?}"),
    };
    assert_eq!(refused(&f), "refs/heads/a..b");
    assert!(!oplog_path(&f.common).exists());
    opened(open_remote_intent(&f.git, &f.common, &remote_op("op-good", &f.objects), "origin").unwrap());
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(refused(&f), "refs/heads/a..b");
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn recover_leaves_the_remote_intent_open_and_settles_the_unmarked_one() {
    // Joins open_remote_intent -> recover's recoverable_intents -> read_marked_log and open_intent.
    let f = Fixture::new();
    let remote = opened(open_remote_intent(&f.git, &f.common, &operation(&f.objects), "origin").unwrap());
    let local = test_op("op-local", "local-caller", vec![RefMove {
        refname: "refs/mortar/b".into(),
        old: Some(f.objects[0].clone()),
        new: f.objects[1].clone(),
    }]);
    append_op_record(&f.common, &intent_record(&local)).unwrap();
    let closed = recover(&f.git, &f.common).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].op_id, "op-local");
    assert_eq!(closed[0].phase, OpPhase::Failed);
    assert_eq!(closed[0].result, Some(OpResult::NotMoved {
        refs: vec![("refs/mortar/b".into(), Some(f.objects[0].clone()))],
    }));
    let log = read_marked_log(&f.common).unwrap();
    let shape: Vec<(&str, OpPhase, Option<&str>)> = log
        .records
        .iter()
        .map(|m| (m.record.op_id.as_str(), m.record.phase, m.remote.as_deref()))
        .collect();
    assert_eq!(shape, [
        ("integration-op-1", OpPhase::Intent, Some("origin")),
        ("op-local", OpPhase::Intent, None),
        ("op-local", OpPhase::Failed, None),
    ]);
    assert_eq!(open_intent(&f.common, "integration-op-1").unwrap(), Some(remote.record.moves));
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[0]));
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert!(recover(&f.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn apply_lines_stay_byte_identical_beside_a_marked_line() {
    // Joins open_remote_intent's marked line -> apply -> the bytes apply appends after it.
    let f = Fixture::new();
    opened(open_remote_intent(&f.git, &f.common, &remote_op("op-remote", &f.objects), "origin").unwrap());
    apply(&f.git, &f.common, &operation(&f.objects)).unwrap();
    let text = fs::read_to_string(oplog_path(&f.common)).unwrap();
    let first = text.lines().next().unwrap().len() + 1;
    // The done record's `at` is the clock when it closed, so it is taken from the decoded record.
    let at = read_op_log(&f.common).unwrap().records[2].at;
    let (head, moves) = operation_text(&f.objects);
    let expected = format!(
        "{head},\"at\":100,\"phase\":\"intent\",\"moves\":{moves}}}\n\
         {head},\"at\":{at},\"phase\":\"done\",\"moves\":{moves},\"result\":{{\"outcome\":\"moved\"}}}}\n"
    );
    assert_eq!(&text[first..], expected);
    assert!(!text[first..].contains("\"remote\""));
}
