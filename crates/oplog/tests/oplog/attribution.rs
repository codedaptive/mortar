//! #1179/S-60: apply/apply_protected/op_git -> real transaction/pre-push hooks; open_intent -> durable real log.
use crate::support::{operation, set_ref, Fixture};
use gitio::{Git, RunOpts};
use oplog::{append_op_record, apply, apply_protected, close_record, intent_record, open_intent, op_git,
    oplog_path, read_op_log, ApplyOutcome, OpPhase, OpResult, OplogError, RefMove};
use std::{fs::{self, OpenOptions}, io::Write};
use testkit::write_git_shim;

#[test]
fn apply_and_protected_apply_hooks_see_the_operation_id_without_mutating_the_callers_git() {
    for protected in [false, true] {
        let f = Fixture::new();
        let trace = f.common.join("transaction.trace");
        write_git_shim(&f.common.join("hooks/reference-transaction"), b"#!/bin/sh\nset -eu\n# Observe the two transaction phases supported throughout our Git version range.\ncase \"$1\" in prepared|committed) ;; *) cat >/dev/null; exit 0;; esac\nprintf 'state:%s op:%s\\n' \"$1\" \"${MORTAR_OP-UNSET}\" >>\"$MORTAR_TEST_HOOK_TRACE\"\ncat >>\"$MORTAR_TEST_HOOK_TRACE\"\n").unwrap();
        let mut caller = f.git.clone();
        caller.env.extend([("MORTAR_OP".into(), "caller-original".into()), ("MORTAR_TEST_HOOK_TRACE".into(), trace.clone().into_os_string())]);
        let original = caller.env.clone();
        let op = operation(&f.objects);
        let answer = if protected { apply_protected(&caller, &f.common, &op, &[]) } else { apply(&caller, &f.common, &op) }.unwrap();
        assert!(matches!(answer, ApplyOutcome::Done(_)));
        let pair = format!("{} {} refs/mortar/a\n{} {} refs/mortar/b\n", f.objects[0], f.objects[1], f.objects[0], f.objects[1]);
        assert_eq!(fs::read_to_string(&trace).unwrap(), format!("state:prepared op:integration-op-1\n{pair}state:committed op:integration-op-1\n{pair}"));
        assert_eq!(caller.env, original);
        assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), None);
        // A subsequent transaction through the ORIGINAL caller sees only its prior value, never the apply id.
        set_ref(&caller, "refs/mortar/unattributed", &f.objects[2]);
        let contents = fs::read_to_string(&trace).unwrap();
        assert_eq!(contents.matches("op:integration-op-1").count(), 2);
        assert_eq!(contents.matches("op:caller-original").count(), 2);
        let records = read_op_log(&f.common).unwrap().records;
        assert_eq!(records.len(), 2); assert_eq!(records[1].result, Some(OpResult::Moved));
    }
}

#[test]
fn real_open_intent_log_reads_exact_moves_and_done_or_failed_closes_only_that_intent() {
    for failed in [false, true] {
        let f = Fixture::new();
        let op = operation(&f.objects);
        assert!(!oplog_path(&f.common).exists());
        assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), None);
        assert!(!oplog_path(&f.common).exists(), "lookup does not create a missing log");
        let intent = intent_record(&op);
        append_op_record(&f.common, &intent).unwrap();
        assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), Some(vec![
            RefMove { refname: "refs/mortar/a".into(), old: Some(f.objects[0].clone()), new: f.objects[1].clone() },
            RefMove { refname: "refs/mortar/b".into(), old: Some(f.objects[0].clone()), new: f.objects[1].clone() },
        ]));
        assert_eq!(open_intent(&f.common, "absent-op").unwrap(), None);
        let mut other = op.clone(); other.op_id = "other-open".into(); other.key = "other-key".into();
        append_op_record(&f.common, &intent_record(&other)).unwrap();
        let (phase, result) = if failed {
            (OpPhase::Failed, OpResult::Stale { refname: "refs/mortar/a".into(), actual: Some(f.objects[2].clone()) })
        } else { (OpPhase::Done, OpResult::Moved) };
        append_op_record(&f.common, &close_record(&intent, phase, result, 101)).unwrap();
        assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), None);
        assert_eq!(open_intent(&f.common, "other-open").unwrap(), Some(other.moves));
    }
}

#[test]
fn torn_tail_does_not_hide_an_earlier_complete_intent_and_append_after_it_remains_readable() {
    let f = Fixture::new();
    let op = operation(&f.objects);
    append_op_record(&f.common, &intent_record(&op)).unwrap();
    let path = oplog_path(&f.common);
    OpenOptions::new().append(true).open(&path).unwrap().write_all(b"{\"op_id\":\"torn").unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), Some(op.moves.clone()));
    assert_eq!(open_intent(&f.common, "torn").unwrap(), None);
    assert_eq!(fs::read(&path).unwrap(), bytes, "lookup must not repair or truncate the log");
    let mut second = op.clone(); second.op_id = "later-open".into(); second.key = "later-key".into();
    append_op_record(&f.common, &intent_record(&second)).unwrap();
    assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), Some(op.moves));
    assert_eq!(open_intent(&f.common, &second.op_id).unwrap(), Some(second.moves));
    let log = read_op_log(&f.common).unwrap();
    assert_eq!(log.records.len(), 2); assert_eq!(log.torn_lines.len(), 1);
}

#[test]
fn unreadable_log_path_is_a_typed_io_error_naming_the_actual_path() {
    let f = Fixture::new();
    let path = oplog_path(&f.common);
    fs::create_dir_all(&path).unwrap(); // Deterministic failure even when the test user can bypass permissions.
    let OplogError::Io { path: actual, source } = open_intent(&f.common, "op").unwrap_err() else { panic!("expected Io") };
    assert_eq!(actual, path); assert_ne!(source.kind(), std::io::ErrorKind::NotFound);
    assert!(path.is_dir());
}

#[test]
fn a_real_push_inside_an_open_intent_carries_op_git_id_to_pre_push_and_then_closes() {
    let f = Fixture::new();
    let remote = f.repo.path.join("remote.git");
    f.git.run(&["init", "--bare", remote.to_str().unwrap()], &RunOpts::default()).unwrap();
    let trace = f.common.join("pre-push.trace");
    write_git_shim(&f.common.join("hooks/pre-push"), b"#!/bin/sh\nset -eu\nprintf 'op:%s\\n' \"${MORTAR_OP-UNSET}\" >\"$MORTAR_TEST_PUSH_TRACE\"\ncat >>\"$MORTAR_TEST_PUSH_TRACE\"\n").unwrap();
    let mut caller = f.git.clone();
    caller.env.extend([("MORTAR_OP".into(), "caller-original".into()), ("MORTAR_TEST_PUSH_TRACE".into(), trace.clone().into_os_string())]);
    let mut op = operation(&f.objects); op.op_id = "local-push-op".into(); op.key = "local-push-key".into();
    op.moves = vec![RefMove { refname: "refs/mortar/pushed".into(), old: None, new: f.objects[0].clone() }];
    let intent = intent_record(&op); append_op_record(&f.common, &intent).unwrap();
    assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), Some(op.moves.clone()));
    let original = caller.env.clone();
    op_git(&caller, &op.op_id).run(&["push", remote.to_str().unwrap(), "refs/mortar/a:refs/mortar/pushed"], &RunOpts::default()).unwrap();
    assert_eq!(fs::read_to_string(&trace).unwrap(), format!("op:local-push-op\nrefs/mortar/a {} refs/mortar/pushed {}\n", f.objects[0], "0".repeat(40)));
    assert_eq!(caller.env, original);
    let remote_git = Git { repo: remote, ..f.git.clone() };
    assert_eq!(remote_git.read_ref("refs/mortar/pushed").unwrap(), Some(f.objects[0].clone()));
    append_op_record(&f.common, &close_record(&intent, OpPhase::Done, OpResult::Moved, 102)).unwrap();
    assert_eq!(open_intent(&f.common, &op.op_id).unwrap(), None);
    assert_eq!(f.git.read_ref("refs/mortar/a").unwrap(), Some(f.objects[0].clone()));
}
