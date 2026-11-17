use super::non_read_git_calls;

#[test]
fn writes_from_mortar_processes_are_not_listed() {
    let log = "10\t9\tmr\tgit\t-C\t/w\tupdate-ref\trefs/heads/zone/a\tabc\n\
               11\t9\tmortar\tgit\tcommit-tree\tabc\n";
    assert_eq!(non_read_git_calls(log), Vec::<String>::new());
}

#[test]
fn reads_from_the_agent_are_not_listed() {
    let log = "20\t19\tsh\tgit\tstatus\n21\t19\tnode\tgit\t-C\t/w\tdiff\n22\t19\tzsh\tgit\tlog\t--oneline\n";
    assert_eq!(non_read_git_calls(log), Vec::<String>::new());
}

#[test]
fn a_write_from_the_agent_is_listed_with_its_parent() {
    let log = "20\t19\tsh\tgit\tstatus\n\
               21\t19\tsh\tgit\tcommit\t--allow-empty\t-m\tx\n\
               22\t9\tmr\tgit\tcommit-tree\tabc\n\
               23\t19\tbash\tgit\tpush\torigin\tmain\n";
    assert_eq!(
        non_read_git_calls(log),
        vec!["git commit --allow-empty -m x (parent sh)".to_string(), "git push origin main (parent bash)".to_string()]
    );
}

#[test]
fn a_call_whose_parent_is_git_is_judged_like_the_agents() {
    assert_eq!(non_read_git_calls("30\t29\tgit\tgit\tgc\t--auto\n"), vec!["git gc --auto (parent git)".to_string()]);
}

#[test]
fn an_unreadable_line_is_listed_and_blank_lines_are_skipped() {
    assert_eq!(non_read_git_calls("\n  \nnot a log line\n"), vec!["unreadable git.log line: not a log line".to_string()]);
}
