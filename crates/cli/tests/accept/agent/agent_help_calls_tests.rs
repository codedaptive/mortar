use super::agent_help_calls;

#[test]
fn the_agents_help_and_version_calls_are_listed_in_log_order() {
    let log = "1\t0\tsh\tmr\thelp\n2\t0\tsh\tmr\tstatus\n3\t0\tnode\tmr\tland\t--help\n4\t0\tsh\tmortar\t--version\n";
    assert_eq!(
        agent_help_calls(log),
        vec![
            "mr help (parent sh)".to_string(),
            "mr land --help (parent node)".to_string(),
            "mortar --version (parent sh)".to_string(),
        ]
    );
}

#[test]
fn a_drives_or_a_hooks_help_call_is_not_listed() {
    let log = "1\t1\tdrive\tmr\thelp\n2\t0\tgit\tmortar\t--help\n";
    assert_eq!(agent_help_calls(log), Vec::<String>::new());
}

#[test]
fn the_six_verbs_are_not_listed() {
    let log = "1\t0\tsh\tmr\tstatus\n2\t0\tsh\tmr\tsave\tr1: work\n3\t0\tsh\tmr\tjoin\n4\t0\tsh\tmr\tresolve\n\
               5\t0\tsh\tmortar\tneed\tb\n6\t0\tsh\tmr\tundo\n";
    assert_eq!(agent_help_calls(log), Vec::<String>::new());
}

#[test]
fn blank_and_unreadable_lines_are_skipped() {
    assert_eq!(agent_help_calls("\nmr help\n1\t0\tsh\tmr\t-h\n"), vec!["mr -h (parent sh)".to_string()]);
}
