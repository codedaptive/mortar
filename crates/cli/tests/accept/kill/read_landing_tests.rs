use super::{read_landing, LandingSeen};

#[test]
fn reads_each_member_of_the_landing_state() {
    let text = "{\"change_id\":\"Iabc\",\"zone\":\"b\",\"squash\":\"s1\",\"origin_at_start\":\"o1\",\"integration\":\"s1\"}\n";
    assert_eq!(
        read_landing(text),
        Ok(LandingSeen {
            change_id: "Iabc".to_string(),
            zone: "b".to_string(),
            squash: "s1".to_string(),
            origin_at_start: "o1".to_string(),
            integration: "s1".to_string(),
        })
    );
}

#[test]
fn a_missing_member_is_named() {
    let text = "{\"change_id\":\"Iabc\",\"zone\":\"b\",\"squash\":\"s1\",\"integration\":\"s1\"}";
    assert_eq!(read_landing(text), Err("landing.json has no string origin_at_start".to_string()));
}

#[test]
fn a_member_of_another_kind_is_named() {
    let text = "{\"change_id\":7,\"zone\":\"b\",\"squash\":\"s1\",\"origin_at_start\":\"o1\",\"integration\":\"s1\"}";
    assert_eq!(read_landing(text), Err("landing.json has no string change_id".to_string()));
}

#[test]
fn text_that_is_not_json_is_an_error() {
    assert!(read_landing("not json").is_err());
}
