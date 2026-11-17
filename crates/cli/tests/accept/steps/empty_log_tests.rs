use super::empty_log;

const EMPTY: &str = "{\"verb\":\"log\",\"ok\":true,\"outcome\":\"empty\",\"exit\":0,\
                     \"next\":\"mr log with no filter lists every op\",\"lines\":[],\"owner\":null,\"records\":[]}\n";

#[test]
#[ignore = "acceptance: make accept"]
fn the_real_empty_report_passes() {
    assert_eq!(empty_log(Some(0), EMPTY), (true, "exit 0; outcome empty; 0 records".to_string()));
}

#[test]
#[ignore = "acceptance: make accept"]
fn records_in_the_list_fail() {
    let stdout = r#"{"verb":"log","outcome":"empty","records":[{"op":"1"},{"op":"2"}]}"#;
    assert_eq!(empty_log(Some(0), stdout), (false, "exit 0; outcome empty; 2 records".to_string()));
}

#[test]
#[ignore = "acceptance: make accept"]
fn exactly_one_record_fails() {
    let stdout = r#"{"verb":"log","ok":true,"outcome":"empty","exit":0,"next":"x","lines":[],"owner":null,"records":[{"op":"1"}]}"#;
    assert_eq!(empty_log(Some(0), stdout), (false, "exit 0; outcome empty; 1 record".to_string()));
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_listed_outcome_fails() {
    let stdout = r#"{"verb":"log","outcome":"listed","records":[]}"#;
    assert_eq!(empty_log(Some(0), stdout), (false, "exit 0; outcome listed; 0 records".to_string()));
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_non_zero_exit_fails() {
    assert_eq!(empty_log(Some(1), EMPTY), (false, "exit 1; outcome empty; 0 records".to_string()));
    assert_eq!(empty_log(None, EMPTY), (false, "no exit code; outcome empty; 0 records".to_string()));
}

#[test]
#[ignore = "acceptance: make accept"]
fn stdout_that_is_not_json_fails() {
    assert_eq!(
        empty_log(Some(0), "log: nothing\n"),
        (false, "exit 0; stdout is not JSON: expected a value at byte 0".to_string())
    );
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_missing_or_wrongly_typed_field_fails() {
    assert_eq!(
        empty_log(Some(0), r#"{"verb":"log","outcome":"empty"}"#),
        (false, "exit 0; outcome empty; no records field".to_string())
    );
    assert_eq!(
        empty_log(Some(0), r#"{"outcome":"empty","records":{}}"#),
        (false, "exit 0; outcome empty; records is not an array".to_string())
    );
    assert_eq!(empty_log(Some(0), r#"{"records":[]}"#), (false, "exit 0; no outcome field; 0 records".to_string()));
}
