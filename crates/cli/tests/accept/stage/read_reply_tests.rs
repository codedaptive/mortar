use super::{read_reply, test_ran as ran};
use crate::json::JsonValue;

const JOINED: &str = "{\"verb\":\"join\",\"ok\":true,\"outcome\":\"joined\",\"exit\":0,\
                      \"next\":\"wait for the zone's tests\",\"lines\":[\"saved\",\"joined: zone moved\"],\
                      \"owner\":null,\"to\":\"abc\"}\n";

#[test]
fn a_report_gives_its_outcome_next_and_lines() {
    let reply = read_reply(&ran(Some(0), JOINED));
    assert_eq!(reply.exit, Some(0));
    assert_eq!(reply.outcome, "joined");
    assert_eq!(reply.next, "wait for the zone's tests");
    assert_eq!(reply.lines, vec!["saved".to_string(), "joined: zone moved".to_string()]);
    assert_eq!(reply.saw, "exit 0; outcome joined");
}

#[test]
fn a_run_that_did_not_exit_0_shows_the_first_line() {
    let stdout = r#"{"verb":"join","outcome":"conflicted","exit":2,"next":"mr resolve","lines":["conflicted: commit c","  src/a/a.txt"]}"#;
    let reply = read_reply(&ran(Some(2), stdout));
    assert_eq!(reply.saw, "exit 2; outcome conflicted; conflicted: commit c");
    assert_eq!(reply.next, "mr resolve");
}

#[test]
fn a_failing_report_with_no_lines_shows_the_outcome_only() {
    let reply = read_reply(&ran(Some(1), r#"{"outcome":"refused","lines":[]}"#));
    assert_eq!(reply.saw, "exit 1; outcome refused");
}

#[test]
fn stdout_that_is_not_json_gives_an_empty_reply_naming_the_error() {
    let reply = read_reply(&ran(Some(4), "usage: mr join\n"));
    assert_eq!(reply.outcome, "");
    assert_eq!(reply.next, "");
    assert!(reply.lines.is_empty());
    assert_eq!(reply.report, JsonValue::Null);
    assert_eq!(reply.saw, "exit 4; stdout is not JSON: expected a value at byte 0");
}

#[test]
fn a_child_that_did_not_run_is_not_json_either() {
    let reply = read_reply(&ran(None, ""));
    assert_eq!(reply.exit, None);
    assert_eq!(reply.saw, "did not run; stdout is not JSON: expected a value at the end of the text");
}
