use crate::support::{operation, refs, set_ref, Fixture};
use gitio::{Git, Oid};
use oplog::{
    apply, apply_with_values, oplog_path, read_op_log, read_valued_log, ApplyOutcome, OpPhase, OpResult,
    ValuedOutcome,
};
use std::fs;

/// Caller-set values: a repeated name, an empty value and one needing JSON escapes, in this order.
pub fn pairs() -> Vec<(String, String)> {
    [
        ("folder_tree", "4b825dc642cb6eb9a060e54bf8d69288fbee4904"),
        ("note", "say \"hi\"\\back\nline two"),
        ("folder_tree", ""),
    ]
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .to_vec()
}

#[test]
fn apply_with_values_writes_intent_and_done_carrying_the_values() {
    // Joins apply_with_values -> Git::update_refs -> append_valued_record -> read_valued_log/read_op_log.
    let f = Fixture::new();
    let got = apply_with_values(&f.git, &f.common, &operation(&f.objects), &pairs()).unwrap();
    let done = match got.outcome {
        ApplyOutcome::Done(record) => record,
        other => panic!("expected Done: {other:?}"),
    };
    assert_eq!(got.values, pairs());
    assert_eq!(done.result, Some(OpResult::Moved));
    assert_eq!(refs(&f), format!("refs/mortar/a {0}\nrefs/mortar/b {0}\n", f.objects[1]));
    let log = read_valued_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    let phases: Vec<OpPhase> = log.records.iter().map(|v| v.record.phase).collect();
    assert_eq!(phases, [OpPhase::Intent, OpPhase::Done]);
    for valued in &log.records {
        assert_eq!(valued.values, pairs());
    }
    assert_eq!(log.records[1].record, done);
    let plain: Vec<_> = log.records.iter().map(|v| v.record.clone()).collect();
    assert_eq!(read_op_log(&f.common).unwrap().records, plain);
    let text = fs::read_to_string(oplog_path(&f.common)).unwrap();
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains(r#"{"name":"note","value":"say \"hi\"\\back\nline two"}"#), "{text}");
}

#[test]
fn repeat_with_other_values_answers_the_first_values_and_appends_nothing() {
    // Joins apply_with_values' repeat check -> read_valued_log on the first run's done record.
    let f = Fixture::new();
    let op = operation(&f.objects);
    let first = apply_with_values(&f.git, &f.common, &op, &pairs()).unwrap();
    assert!(matches!(first.outcome, ApplyOutcome::Done(_)));
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    let second = vec![("folder_tree".to_string(), "second-run".to_string())];
    let again = apply_with_values(&f.git, &f.common, &op, &second).unwrap();
    let done = read_op_log(&f.common).unwrap().records[1].clone();
    assert_eq!(again, ValuedOutcome {
        outcome: ApplyOutcome::Repeated(done),
        values: pairs(),
    });
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn old_apply_writes_lines_without_values_byte_for_byte() {
    // Joins apply -> apply_with_values with none -> the log file's bytes.
    let f = Fixture::new();
    apply(&f.git, &f.common, &operation(&f.objects)).unwrap();
    let text = fs::read_to_string(oplog_path(&f.common)).unwrap();
    let at = closing_at(&text);
    let (old, new) = (&f.objects[0], &f.objects[1]);
    let moves = format!(
        r#"[{{"ref":"refs/mortar/a","old":"{old}","new":"{new}"}},{{"ref":"refs/mortar/b","old":"{old}","new":"{new}"}}]"#
    );
    let head = r#"{"op_id":"integration-op-1","key":"integration-repeat-key","kind":"join","actor":"integration-actor","reason":"move two refs atomically""#;
    let expected = format!(
        "{head},\"at\":100,\"phase\":\"intent\",\"moves\":{moves}}}\n\
         {head},\"at\":{at},\"phase\":\"done\",\"moves\":{moves},\"result\":{{\"outcome\":\"moved\"}}}}\n"
    );
    assert_eq!(text, expected);
}

/// The done record's `at` is the clock when it closed, so it is read from the file's second line as text.
fn closing_at(text: &str) -> &str {
    let line = text.lines().nth(1).unwrap();
    let rest = &line[line.find("\"at\":").unwrap() + "\"at\":".len()..];
    &rest[..rest.find(',').unwrap()]
}

#[test]
fn stale_apply_with_values_closes_failed_keeping_the_values() {
    // Joins apply_with_values -> Git::update_refs stale -> read_ref_value -> the failed valued record.
    let f = Fixture::new();
    set_ref(&f.git, "refs/mortar/b", &f.objects[2]);
    let got = apply_with_values(&f.git, &f.common, &operation(&f.objects), &pairs()).unwrap();
    let failed = match got.outcome {
        ApplyOutcome::Failed(record) => record,
        other => panic!("expected Failed: {other:?}"),
    };
    assert_eq!(failed.result, Some(OpResult::Stale {
        refname: "refs/mortar/b".into(),
        actual: Some(f.objects[2].clone()),
    }));
    assert_eq!(got.values, pairs());
    let records = read_valued_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].record.phase, OpPhase::Intent);
    assert_eq!(records[1].record, failed);
    assert_eq!(records[0].values, pairs());
    assert_eq!(records[1].values, pairs());
    assert_eq!(refs(&f), format!("refs/mortar/a {}\nrefs/mortar/b {}\n", f.objects[0], f.objects[2]));
}

#[test]
#[ignore = "SIGKILL subprocess entry point; exercised explicitly by values_recover's kill test"]
fn apply_values_worker() {
    let path = std::path::PathBuf::from(std::env::var_os("MORTAR_TEST_REPO").unwrap());
    let objects = ["MORTAR_TEST_OLD", "MORTAR_TEST_NEW", "MORTAR_TEST_FOREIGN"]
        .map(|key| Oid::parse(&std::env::var(key).unwrap()).unwrap());
    let mut git = Git::new(&path);
    git.program = path.join("git-crash-boundary.sh");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    apply_with_values(&git, &path.join(".git"), &operation(&objects), &pairs()).unwrap();
    panic!("crash shim returned without killing apply_with_values");
}
