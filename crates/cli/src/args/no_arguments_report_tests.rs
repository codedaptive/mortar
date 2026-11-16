//! no_arguments_report: nothing for no arguments; a usage report naming the verb for any argument.

use super::no_arguments_report;
use crate::report::ExitCode;

#[test]
fn no_arguments_is_no_report() {
    assert_eq!(no_arguments_report("status", &[], false), None);
}

#[test]
fn one_argument_is_a_usage_report_naming_the_verb_and_its_usage_line() {
    let report = no_arguments_report("status", &["extra".to_string()], false).expect("an argument is refused");
    assert_eq!(report.verb, "status");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.lines[0], "status takes no arguments");
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "mortar status [--json]");
    assert!(report.fields.is_empty());
}

#[test]
fn json_asked_for_gives_the_same_usage_report() {
    let report =
        no_arguments_report("resolve", &["a".to_string(), "b".to_string()], true).expect("arguments are refused");
    assert_eq!(report.verb, "resolve");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.lines[0], "resolve takes no arguments");
    assert_eq!(report.next, "mortar resolve [--key <operation key>] [--json]");
}
