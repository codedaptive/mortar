//! render_json: one line, keys verb, ok, outcome, exit, next, lines, owner, then the fields in order.

use super::{render_json, ExitCode, OutValue, Report};

fn report(exit: ExitCode, outcome: &str) -> Report {
    Report {
        verb: "init".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: vec!["one".to_string(), "two \"q\"".to_string()],
        owner: None,
        next: "mortar init".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn done_report_with_null_owner() {
    assert_eq!(
        render_json(&report(ExitCode::Done, "written")),
        "{\"verb\":\"init\",\"ok\":true,\"outcome\":\"written\",\"exit\":0,\"next\":\"mortar init\",\
         \"lines\":[\"one\",\"two \\\"q\\\"\"],\"owner\":null}\n"
    );
}

#[test]
fn refusal_shows_owner_and_is_not_ok() {
    let mut refused = report(ExitCode::Refused, "refused");
    refused.lines = Vec::new();
    refused.owner = Some("the existing config".to_string());
    assert_eq!(
        render_json(&refused),
        "{\"verb\":\"init\",\"ok\":false,\"outcome\":\"refused\",\"exit\":1,\"next\":\"mortar init\",\
         \"lines\":[],\"owner\":\"the existing config\"}\n"
    );
}

#[test]
fn every_exit_code_is_its_number() {
    let codes = [
        (ExitCode::Done, "0"),
        (ExitCode::Refused, "1"),
        (ExitCode::Conflict, "2"),
        (ExitCode::Busy, "3"),
        (ExitCode::Usage, "4"),
        (ExitCode::Failure, "5"),
    ];
    for (code, number) in codes {
        let text = render_json(&report(code, "x"));
        assert!(text.contains(&format!(",\"exit\":{number},")), "{text}");
    }
}

#[test]
fn fields_follow_owner_in_order_with_each_value_kind() {
    let mut busy = report(ExitCode::Busy, "busy");
    busy.verb = String::new();
    busy.lines = Vec::new();
    busy.fields = vec![
        ("holder_host".to_string(), OutValue::Text("h\n1".to_string())),
        ("holder_pid".to_string(), OutValue::Number(42)),
        ("stale".to_string(), OutValue::Bool(false)),
        ("fresh".to_string(), OutValue::Bool(true)),
        ("key".to_string(), OutValue::Null),
        ("paths".to_string(), OutValue::Texts(vec!["a".to_string(), "b".to_string()])),
    ];
    assert_eq!(
        render_json(&busy),
        "{\"verb\":\"\",\"ok\":false,\"outcome\":\"busy\",\"exit\":3,\"next\":\"mortar init\",\"lines\":[],\
         \"owner\":null,\"holder_host\":\"h\\n1\",\"holder_pid\":42,\"stale\":false,\"fresh\":true,\
         \"key\":null,\"paths\":[\"a\",\"b\"]}\n"
    );
}

#[test]
fn a_raw_field_writes_its_values_as_they_are() {
    let mut listed = report(ExitCode::Done, "listed");
    listed.lines = Vec::new();
    listed.fields = vec![
        ("records".to_string(), OutValue::Raw(vec!["{\"op_id\":\"a\"}".to_string(), "[1,null]".to_string()])),
        ("none".to_string(), OutValue::Raw(Vec::new())),
    ];
    assert_eq!(
        render_json(&listed),
        "{\"verb\":\"init\",\"ok\":true,\"outcome\":\"listed\",\"exit\":0,\"next\":\"mortar init\",\"lines\":[],\
         \"owner\":null,\"records\":[{\"op_id\":\"a\"},[1,null]],\"none\":[]}\n"
    );
}
