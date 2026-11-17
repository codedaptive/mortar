use super::agent_verbs_outside_six;

#[test]
fn the_six_verbs_from_the_agent_are_not_listed() {
    let log = "1\t0\tsh\tmr\tstatus\n2\t0\tsh\tmr\tsave\tr1: work\n3\t0\tsh\tmr\tjoin\n4\t0\tsh\tmr\tresolve\n\
               5\t0\tsh\tmortar\tneed\tb\n6\t0\tsh\tmr\tundo\n7\t0\tsh\tmr\t--json\tstatus\n";
    assert_eq!(agent_verbs_outside_six(log), Vec::<String>::new());
}

#[test]
fn another_verb_from_the_agent_is_listed() {
    let log = "1\t0\tsh\tmr\tstatus\n2\t0\tnode\tmr\tland\tabc\n3\t0\tsh\tmortar\tfolder\trequest\ta\tr9\n";
    assert_eq!(
        agent_verbs_outside_six(log),
        vec!["mr land abc (parent node)".to_string(), "mortar folder request a r9 (parent sh)".to_string()]
    );
}

#[test]
fn hook_calls_and_the_drives_calls_are_not_the_agents() {
    let log = "1\t0\tgit\tmortar\tguard-hook\treference-transaction\n2\t2\tdrive\tmr\tsave\trc: conflict\n\
               3\t2\tdrive\tmortar\tguard-hook\tpre-push\n";
    assert_eq!(agent_verbs_outside_six(log), Vec::<String>::new());
}

#[test]
fn a_call_naming_no_verb_is_not_listed() {
    assert_eq!(agent_verbs_outside_six("1\t0\tsh\tmr\t--help\n"), Vec::<String>::new());
}

#[test]
fn an_unreadable_line_is_listed_and_blank_lines_are_skipped() {
    assert_eq!(agent_verbs_outside_six("\nmr status\n"), vec!["unreadable mr.log line: mr status".to_string()]);
}
