use crate::support::{kill_worker, operation, test_op, Fixture};
use crate::values::pairs;
use oplog::{
    apply, apply_with_values, history, open_intent, oplog_path, read_op_log, read_valued_log,
    recover, ApplyOutcome, HistoryFilter, OpPhase, OpResult, RefMove, ValuedOutcome,
};
use std::fs;

#[test]
fn kill_after_transaction_then_recover_closes_done_keeping_the_values() {
    // Joins a SIGKILLed apply_with_values -> recover -> recovery_close_valued -> read_valued_log -> repeat.
    let f = Fixture::new();
    kill_worker(&f, "after", "values::apply_values_worker");
    let open = read_valued_log(&f.common).unwrap().records;
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].values, pairs());
    let closed = recover(&f.git, &f.common).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].phase, OpPhase::Done);
    assert_eq!(closed[0].result, Some(OpResult::Moved));
    let records = read_valued_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].record, closed[0]);
    assert_eq!(records[1].values, pairs());
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let retry = vec![("folder_tree".to_string(), "retry".to_string())];
    let again = apply_with_values(&f.git, &f.common, &operation(&f.objects), &retry).unwrap();
    assert_eq!(again, ValuedOutcome {
        outcome: ApplyOutcome::Repeated(closed[0].clone()),
        values: pairs(),
    });
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn log_mixing_lines_with_and_without_values_reads_whole() {
    // Joins apply and apply_with_values lines in one file -> every reader of the log.
    let f = Fixture::new();
    let step = |name: &str, old: usize, new: usize| RefMove {
        refname: name.into(),
        old: Some(f.objects[old].clone()),
        new: f.objects[new].clone(),
    };
    apply(&f.git, &f.common, &test_op("op-1", "old-caller", vec![step("refs/mortar/a", 0, 1)])).unwrap();
    let valued = test_op("op-2", "valued-caller", vec![step("refs/mortar/b", 0, 1)]);
    apply_with_values(&f.git, &f.common, &valued, &pairs()).unwrap();
    apply(&f.git, &f.common, &test_op("op-3", "old-caller", vec![step("refs/mortar/a", 1, 2)])).unwrap();
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let log = read_valued_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    let shape: Vec<(&str, OpPhase, usize)> = log
        .records
        .iter()
        .map(|v| (v.record.op_id.as_str(), v.record.phase, v.values.len()))
        .collect();
    assert_eq!(shape, [
        ("op-1", OpPhase::Intent, 0),
        ("op-1", OpPhase::Done, 0),
        ("op-2", OpPhase::Intent, 3),
        ("op-2", OpPhase::Done, 3),
        ("op-3", OpPhase::Intent, 0),
        ("op-3", OpPhase::Done, 0),
    ]);
    assert_eq!(log.records[3].values, pairs());
    let plain = read_op_log(&f.common).unwrap();
    assert!(plain.torn_lines.is_empty());
    let records: Vec<_> = log.records.iter().map(|v| v.record.clone()).collect();
    assert_eq!(plain.records, records);
    let copy = Fixture::new();
    fs::create_dir_all(oplog_path(&copy.common).parent().unwrap()).unwrap();
    fs::write(oplog_path(&copy.common), &bytes).unwrap();
    assert_eq!(read_op_log(&copy.common).unwrap(), plain);
    let filtered = history(&f.common, &HistoryFilter::Actor("valued-caller".into())).unwrap();
    let ids: Vec<String> = filtered.into_iter().map(|r| r.op_id).collect();
    assert_eq!(ids, ["op-2", "op-2"]);
    assert_eq!(open_intent(&f.common, "op-2").unwrap(), None);
    assert!(recover(&f.git, &f.common).unwrap().is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}
