use super::LogCall;

#[test]
fn a_call_is_shown_as_its_argv_and_its_parent() {
    let call = LogCall {
        pid: 9,
        ppid: 8,
        parent: "sh".to_string(),
        args: vec!["git".to_string(), "commit".to_string(), "-m".to_string(), "two words".to_string()],
    };
    assert_eq!(call.shown(), "git commit -m two words (parent sh)");
}
