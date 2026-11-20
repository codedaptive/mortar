use super::{parse_log_line, LogCall};

#[test]
fn a_line_reads_its_pids_parent_and_argv_with_spaces_kept_inside_an_argument() {
    let call = parse_log_line("4242\t4100\tsh\tmr\tsave\tr1: work");
    let expected = LogCall {
        pid: 4242,
        ppid: 4100,
        parent: "sh".to_string(),
        args: vec!["mr".to_string(), "save".to_string(), "r1: work".to_string()],
    };
    assert_eq!(call, Some(expected));
}

#[test]
fn a_line_with_only_argv_zero_is_read() {
    let call = parse_log_line("7\t1\tgit\tmortar");
    assert_eq!(call.map(|call| call.args), Some(vec!["mortar".to_string()]));
}

#[test]
fn a_line_without_argv_or_with_a_bad_pid_or_an_empty_parent_is_none() {
    assert_eq!(parse_log_line("7\t1\tsh"), None);
    assert_eq!(parse_log_line("x\t1\tsh\tgit\tstatus"), None);
    assert_eq!(parse_log_line("7\t\tsh\tgit\tstatus"), None);
    assert_eq!(parse_log_line("7\t1\t\tgit\tstatus"), None);
    assert_eq!(parse_log_line("7 1 sh git status"), None);
    assert_eq!(parse_log_line(""), None);
}
