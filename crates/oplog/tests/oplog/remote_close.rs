use crate::remote_intents::remote_op;
use crate::support::{refs, test_op, Fixture};
use crate::values::pairs;
use oplog::{
    append_marked_record, apply, apply_with_values, close_intent, decode_op_log, intent_record, open_intent,
    open_remote_intent, oplog_path, read_marked_log, read_op_log, read_valued_log, IntentClose, OpPhase,
    OpResult, OplogError, RefMove,
};
use std::fs;

fn line_count(f: &Fixture) -> usize {
    fs::read_to_string(oplog_path(&f.common)).unwrap().lines().count()
}

#[test]
fn close_intent_done_then_failed_each_append_one_closing_record_keeping_values_and_mark() {
    // Joins open_remote_intent / append_marked_record -> close_intent -> read_marked_log.
    let f = Fixture::new();
    open_remote_intent(&f.git, &f.common, &remote_op("op-done", &f.objects), "origin").unwrap();
    append_marked_record(&f.common, &intent_record(&remote_op("op-failed", &f.objects)), &pairs(), Some("upstream"))
        .unwrap();
    assert_eq!(line_count(&f), 2);
    let done = close_intent(&f.common, "op-done", IntentClose::Done).unwrap();
    assert_eq!(line_count(&f), 3);
    let refs_given = vec![
        ("refs/heads/main".to_string(), Some(f.objects[2].clone())),
        ("refs/heads/gone".to_string(), None),
    ];
    let failed = close_intent(&f.common, "op-failed", IntentClose::Failed { refs: refs_given.clone() }).unwrap();
    assert_eq!(line_count(&f), 4);
    assert_eq!((done.record.phase, done.record.result.clone()), (OpPhase::Done, Some(OpResult::Moved)));
    assert_eq!((done.remote.as_deref(), done.values.len()), (Some("origin"), 0));
    assert_eq!(failed.record.phase, OpPhase::Failed);
    assert_eq!(failed.record.result, Some(OpResult::NotMoved { refs: refs_given }));
    assert_eq!((failed.remote.as_deref(), failed.values.clone()), (Some("upstream"), pairs()));
    let log = read_marked_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records[2], done);
    assert_eq!(log.records[3], failed);
    assert_eq!(done.record.op_id, "op-done");
    assert_eq!(done.record.moves, remote_op("op-done", &f.objects).moves);
    assert_eq!(open_intent(&f.common, "op-failed").unwrap(), None);
    let text = fs::read_to_string(oplog_path(&f.common)).unwrap();
    assert!(text.lines().nth(2).unwrap().ends_with(r#""result":{"outcome":"moved"},"remote":"origin"}"#), "{text}");
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[0]));
}

#[test]
fn close_intent_refuses_unknown_and_closed_op_ids_appending_nothing() {
    // Joins close_intent's close_intent_record refusal -> the log file left as it was.
    let f = Fixture::new();
    open_remote_intent(&f.git, &f.common, &remote_op("op-remote", &f.objects), "origin").unwrap();
    close_intent(&f.common, "op-remote", IntentClose::Done).unwrap();
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    for op_id in ["op-never-written", "op-remote"] {
        match close_intent(&f.common, op_id, IntentClose::Failed { refs: vec![] }) {
            Err(OplogError::NotOpen { op_id: named }) => assert_eq!(named, op_id),
            other => panic!("expected NotOpen for {op_id}: {other:?}"),
        }
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    }
    assert_eq!(String::from_utf8(bytes).unwrap().lines().count(), 2);
}

#[test]
fn log_mixing_old_and_marked_lines_reads_whole() {
    // Joins apply, apply_with_values, open_remote_intent and close_intent lines in one file -> every reader.
    let f = Fixture::new();
    let step = |name: &str, old: usize, new: usize| RefMove {
        refname: name.into(),
        old: Some(f.objects[old].clone()),
        new: f.objects[new].clone(),
    };
    apply(&f.git, &f.common, &test_op("op-1", "old-caller", vec![step("refs/mortar/a", 0, 1)])).unwrap();
    let valued = test_op("op-2", "valued-caller", vec![step("refs/mortar/b", 0, 1)]);
    apply_with_values(&f.git, &f.common, &valued, &pairs()).unwrap();
    open_remote_intent(&f.git, &f.common, &remote_op("op-3", &f.objects), "origin").unwrap();
    close_intent(&f.common, "op-3", IntentClose::Done).unwrap();
    apply(&f.git, &f.common, &test_op("op-4", "old-caller", vec![step("refs/mortar/a", 1, 2)])).unwrap();
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let marked = read_marked_log(&f.common).unwrap();
    assert!(marked.torn_lines.is_empty());
    let shape: Vec<(&str, OpPhase, usize, Option<&str>)> = marked
        .records
        .iter()
        .map(|m| (m.record.op_id.as_str(), m.record.phase, m.values.len(), m.remote.as_deref()))
        .collect();
    assert_eq!(shape, [
        ("op-1", OpPhase::Intent, 0, None),
        ("op-1", OpPhase::Done, 0, None),
        ("op-2", OpPhase::Intent, 3, None),
        ("op-2", OpPhase::Done, 3, None),
        ("op-3", OpPhase::Intent, 0, Some("origin")),
        ("op-3", OpPhase::Done, 0, Some("origin")),
        ("op-4", OpPhase::Intent, 0, None),
        ("op-4", OpPhase::Done, 0, None),
    ]);
    let records: Vec<_> = marked.records.iter().map(|m| m.record.clone()).collect();
    let valued_log = read_valued_log(&f.common).unwrap();
    assert!(valued_log.torn_lines.is_empty());
    let valued_records: Vec<_> = valued_log.records.iter().map(|v| v.record.clone()).collect();
    assert_eq!(valued_records, records);
    assert_eq!(valued_log.records[3].values, pairs());
    let plain = read_op_log(&f.common).unwrap();
    assert!(plain.torn_lines.is_empty());
    assert_eq!(plain.records, records);
    assert_eq!(decode_op_log(&bytes).unwrap(), plain);
}
