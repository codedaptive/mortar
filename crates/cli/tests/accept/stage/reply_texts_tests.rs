use super::{read_reply, test_ran};

#[test]
fn an_array_of_strings_is_its_items_in_order() {
    let reply = read_reply(&test_ran(Some(2), r#"{"files":["src/a/a.txt","src/a/r1.txt"]}"#));
    assert_eq!(reply.texts("files"), vec!["src/a/a.txt".to_string(), "src/a/r1.txt".to_string()]);
}

#[test]
fn items_that_are_not_strings_are_left_out() {
    let reply = read_reply(&test_ran(Some(2), r#"{"files":["x",1,null,"y"]}"#));
    assert_eq!(reply.texts("files"), vec!["x".to_string(), "y".to_string()]);
}

#[test]
fn a_missing_or_non_array_member_is_none() {
    let reply = read_reply(&test_ran(Some(2), r#"{"files":"src/a/a.txt"}"#));
    assert!(reply.texts("files").is_empty());
    assert!(reply.texts("absent").is_empty());
}
