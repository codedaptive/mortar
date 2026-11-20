//! report_savings: the first Saved field's value, or None.

use super::{report_savings, Savings};
use crate::report::{ExitCode, OutValue, Report};

fn report(fields: Vec<(String, OutValue)>) -> Report {
    Report {
        verb: "status".to_string(),
        exit: ExitCode::Done,
        outcome: "clean".to_string(),
        lines: Vec::new(),
        owner: None,
        next: "x".to_string(),
        fields,
    }
}

fn savings(baseline: &str) -> Savings {
    Savings { baseline: baseline.to_string(), baseline_chars: 9 }
}

#[test]
fn none_without_a_saved_field() {
    assert_eq!(report_savings(&report(vec![("saved".to_string(), OutValue::Bool(true))])), None);
    assert_eq!(report_savings(&report(Vec::new())), None);
}

#[test]
fn the_first_saved_field() {
    let fields = vec![
        ("saved".to_string(), OutValue::Bool(true)),
        ("one".to_string(), OutValue::Saved(savings("first"))),
        ("two".to_string(), OutValue::Saved(savings("second"))),
    ];
    assert_eq!(report_savings(&report(fields)), Some(&savings("first")));
}
