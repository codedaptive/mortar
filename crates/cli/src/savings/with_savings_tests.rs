//! with_savings: a Done report gains ("savings", Saved) last; any other exit comes back unchanged.

use super::{with_savings, Savings};
use crate::report::{ExitCode, OutValue, Report};

fn report(exit: ExitCode) -> Report {
    Report {
        verb: "status".to_string(),
        exit,
        outcome: "clean".to_string(),
        lines: vec!["a".to_string()],
        owner: None,
        next: "x".to_string(),
        fields: vec![("saved".to_string(), OutValue::Bool(false))],
    }
}

fn savings() -> Savings {
    Savings { baseline: "git status --short over this folder".to_string(), baseline_chars: 120 }
}

#[test]
fn done_appends_savings_last() {
    let mut expected = report(ExitCode::Done);
    expected.fields.push(("savings".to_string(), OutValue::Saved(savings())));
    assert_eq!(with_savings(report(ExitCode::Done), savings()), expected);
}

#[test]
fn any_other_exit_is_unchanged() {
    for exit in [ExitCode::Refused, ExitCode::Conflict, ExitCode::Busy, ExitCode::Usage, ExitCode::Failure] {
        assert_eq!(with_savings(report(exit), savings()), report(exit));
    }
}
