//! with_counted_savings: a Done report of a verb with a baseline gains the summed count; a refusal or a verb without a
//! baseline never counts; a failed count or an overflowing sum drops the savings.
//! Unit test: the counters are fakes, so no git runs.

use std::cell::Cell;

use gitio::{BaselineQuery, GitError};

use super::{with_counted_savings, Savings};
use crate::report::{ExitCode, OutValue, Report};

fn report(verb: &str, exit: ExitCode) -> Report {
    Report {
        verb: verb.to_string(),
        exit,
        outcome: "clean".to_string(),
        lines: vec!["a".to_string()],
        owner: None,
        next: "x".to_string(),
        fields: vec![("saved".to_string(), OutValue::Bool(false))],
    }
}

fn two_queries() -> Vec<BaselineQuery> {
    vec![BaselineQuery::Status { paths: vec![] }, BaselineQuery::DiffHead { paths: vec![] }]
}

fn failed() -> GitError {
    GitError { args: vec!["diff".to_string()], code: Some(128), stderr: "fatal".to_string() }
}

fn with_status_savings(baseline_chars: u64) -> Report {
    let mut expected = report("status", ExitCode::Done);
    let baseline = "git status and git diff HEAD over this folder".to_string();
    expected.fields.push(("savings".to_string(), OutValue::Saved(Savings { baseline, baseline_chars })));
    expected
}

#[test]
fn done_sums_the_counts_of_its_queries() {
    let count = |query: &BaselineQuery| match query {
        BaselineQuery::Status { .. } => Ok(120),
        _ => Ok(35),
    };
    assert_eq!(
        with_counted_savings(report("status", ExitCode::Done), &two_queries(), &count),
        with_status_savings(155)
    );
}

#[test]
fn a_failed_count_drops_the_savings() {
    let count = |query: &BaselineQuery| match query {
        BaselineQuery::Status { .. } => Ok(120),
        _ => Err(failed()),
    };
    assert_eq!(
        with_counted_savings(report("status", ExitCode::Done), &two_queries(), &count),
        report("status", ExitCode::Done)
    );
}

#[test]
fn an_overflowing_sum_drops_the_savings() {
    let count = |_: &BaselineQuery| Ok(u64::MAX);
    assert_eq!(
        with_counted_savings(report("status", ExitCode::Done), &two_queries(), &count),
        report("status", ExitCode::Done)
    );
}

#[test]
fn a_report_that_is_not_done_never_counts() {
    for exit in [ExitCode::Refused, ExitCode::Conflict, ExitCode::Busy, ExitCode::Usage, ExitCode::Failure] {
        let calls = Cell::new(0);
        let count = |_: &BaselineQuery| {
            calls.set(calls.get() + 1);
            Ok(10)
        };
        assert_eq!(with_counted_savings(report("status", exit), &two_queries(), &count), report("status", exit));
        assert_eq!(calls.get(), 0, "{exit:?}");
    }
}

#[test]
fn a_verb_without_a_baseline_never_counts() {
    let calls = Cell::new(0);
    let count = |_: &BaselineQuery| {
        calls.set(calls.get() + 1);
        Ok(10)
    };
    assert_eq!(
        with_counted_savings(report("log", ExitCode::Done), &two_queries(), &count),
        report("log", ExitCode::Done)
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn no_queries_is_a_baseline_of_zero() {
    let count = |_: &BaselineQuery| Ok(10);
    assert_eq!(with_counted_savings(report("status", ExitCode::Done), &[], &count), with_status_savings(0));
}
