//! log_line: a marked record with values, on an empty log or after a torn tail (then on a line of its own), has
//! its values then its `remote` key last, and ends in '\n'. log_line and encode_marked_record are pure, so this is a unit test
//! (S-166).

use super::{log_line, OpPhase, OpRecord};

#[test]
fn a_mark_goes_after_the_values_on_a_torn_tail() {
    let r = OpRecord {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    };
    let values = vec![("folder_tree".to_string(), "t1".to_string())];
    assert_eq!(
        log_line(&r, &values, Some("origin"), Some(b'x')),
        "\n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}],\
         \"remote\":\"origin\"}\n"
    );
}

#[test]
fn a_mark_goes_after_the_values() {
    let r = OpRecord {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "push".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 7,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    };
    let values = vec![("n".to_string(), "v".to_string())];
    assert_eq!(
        log_line(&r, &values, Some("origin"), None),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"n\",\"value\":\"v\"}],\
         \"remote\":\"origin\"}\n"
    );
}
