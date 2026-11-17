use super::git_call_is_read;

fn argv(text: &str) -> Vec<String> {
    text.split(' ').map(str::to_string).collect()
}

#[test]
fn each_of_the_nine_reads_is_a_read() {
    for read in ["status", "diff", "log", "show", "ls-files", "rev-parse", "blame", "grep", "cat-file"] {
        assert!(git_call_is_read(&argv(&format!("git {read}"))), "{read}");
    }
}

#[test]
fn writes_and_other_subcommands_are_not_reads() {
    for other in ["commit -m x", "push origin main", "update-ref refs/heads/main abc", "checkout -b x", "merge zone/a",
        "rebase main", "reset --hard", "add -A", "stash", "branch -f main", "fetch origin", "config user.name x"]
    {
        assert!(!git_call_is_read(&argv(&format!("git {other}"))), "{other}");
    }
}

#[test]
fn leading_dash_c_options_and_their_values_are_skipped() {
    assert!(git_call_is_read(&argv("git -C /tmp/r1 status --porcelain")));
    assert!(git_call_is_read(&argv("git -c core.pager=cat log --oneline")));
    assert!(git_call_is_read(&argv("git -C /tmp/r1 -c a.b=c --no-pager diff")));
    assert!(!git_call_is_read(&argv("git -C /tmp/r1 commit --allow-empty -m x")));
    assert!(!git_call_is_read(&argv("git -c log=status commit")));
}

#[test]
fn a_dash_c_value_that_names_a_read_is_not_taken_for_the_subcommand() {
    assert!(!git_call_is_read(&argv("git -C status push")));
}

#[test]
fn a_call_with_no_subcommand_is_a_read() {
    assert!(git_call_is_read(&argv("git --version")));
    assert!(git_call_is_read(&argv("git")));
}
