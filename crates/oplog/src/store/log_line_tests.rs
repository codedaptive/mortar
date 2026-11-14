//! log_line: literal lines written by hand for an empty file, a file ending in '\n', a torn tail and a
//! record with values.

use super::{log_line, OpPhase, OpRecord};

fn record() -> OpRecord {
    OpRecord {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    }
}

#[test]
fn empty_file_gets_the_record_and_a_newline() {
    assert_eq!(
        log_line(&record(), &[], None),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn file_ending_in_newline_gets_no_leading_newline() {
    assert_eq!(
        log_line(&record(), &[], Some(b'\n')),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn torn_tail_gets_a_newline_before_the_record() {
    assert_eq!(
        log_line(&record(), &[], Some(b'}')),
        "\n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn values_go_last_on_the_line() {
    let values = [("folder_tree".to_string(), "t1".to_string())];
    assert_eq!(
        log_line(&record(), &values, Some(b'\n')),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}]}\n"
    );
}
