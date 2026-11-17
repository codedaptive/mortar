use super::{open_intents, OpLine};

fn op(op_id: &str, phase: &str) -> OpLine {
    OpLine { op_id: op_id.to_string(), kind: "land".to_string(), phase: phase.to_string(), ..OpLine::default() }
}

fn ids(lines: Vec<&OpLine>) -> Vec<&str> {
    lines.into_iter().map(|line| line.op_id.as_str()).collect()
}

#[test]
fn an_intent_closed_later_by_done_or_failed_is_not_open() {
    let log = [op("a", "intent"), op("b", "intent"), op("a", "done"), op("b", "failed")];
    assert_eq!(ids(open_intents(&log)), Vec::<&str>::new());
}

#[test]
fn intents_with_no_closing_record_are_open_in_log_order() {
    let log = [op("a", "intent"), op("b", "intent"), op("c", "intent"), op("b", "done")];
    assert_eq!(ids(open_intents(&log)), vec!["a", "c"]);
}

#[test]
fn a_closing_record_of_another_op_closes_nothing() {
    let log = [op("a", "intent"), op("x", "done")];
    assert_eq!(ids(open_intents(&log)), vec!["a"]);
}

#[test]
fn a_closing_record_before_the_intent_does_not_close_it() {
    let log = [op("a", "done"), op("a", "intent")];
    assert_eq!(ids(open_intents(&log)), vec!["a"]);
}

#[test]
fn an_empty_log_has_none() {
    assert_eq!(ids(open_intents(&[])), Vec::<&str>::new());
}
