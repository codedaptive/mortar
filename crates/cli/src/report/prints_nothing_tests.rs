//! prints_nothing: only guard-hook's exit 0 (the guard's allow) prints nothing.

use super::{prints_nothing, ExitCode, Report};

fn report(verb: &str, exit: ExitCode) -> Report {
    Report {
        verb: verb.to_string(),
        exit,
        outcome: "allowed".to_string(),
        lines: Vec::new(),
        owner: None,
        next: String::new(),
        fields: Vec::new(),
    }
}

#[test]
fn guard_hook_done_prints_nothing() {
    assert!(prints_nothing(&report("guard-hook", ExitCode::Done)));
}

#[test]
fn guard_hook_refused_usage_and_failure_are_printed() {
    for exit in [ExitCode::Refused, ExitCode::Usage, ExitCode::Failure] {
        assert!(!prints_nothing(&report("guard-hook", exit)), "{exit:?}");
    }
}

#[test]
fn another_verb_done_is_printed() {
    assert!(!prints_nothing(&report("status", ExitCode::Done)));
}
