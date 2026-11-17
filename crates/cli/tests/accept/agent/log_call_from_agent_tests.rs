use super::LogCall;

fn call(parent: &str) -> LogCall {
    LogCall { pid: 2, ppid: 1, parent: parent.to_string(), args: vec!["mr".to_string(), "status".to_string()] }
}

#[test]
fn a_call_from_a_shell_or_an_agent_tool_is_the_agents() {
    assert!(call("sh").from_agent());
    assert!(call("node").from_agent());
    assert!(call("zsh").from_agent());
}

#[test]
fn a_call_from_git_or_from_the_drive_is_not_the_agents() {
    assert!(!call("git").from_agent());
    assert!(!call("drive").from_agent());
}
