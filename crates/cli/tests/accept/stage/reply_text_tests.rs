use super::{read_reply, test_ran};

#[test]
fn a_string_member_is_its_text() {
    let stdout = r#"{"outcome":"squashed","change_id":"kzlymxnwovpuqtrssrtqupvowxnymlzk","commit":"0a1b"}"#;
    let reply = read_reply(&test_ran(Some(0), stdout));
    assert_eq!(reply.text("change_id"), "kzlymxnwovpuqtrssrtqupvowxnymlzk");
    assert_eq!(reply.text("commit"), "0a1b");
}

#[test]
fn a_missing_null_or_other_member_is_empty() {
    let reply = read_reply(&test_ran(Some(0), r#"{"folder":null,"exit":0,"files":["a"]}"#));
    assert_eq!(reply.text("folder"), "");
    assert_eq!(reply.text("exit"), "");
    assert_eq!(reply.text("files"), "");
    assert_eq!(reply.text("absent"), "");
}
