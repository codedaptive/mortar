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

#[test]
fn run_two_remote_and_config_reads_are_reads_with_or_without_dash_c() {
    for read in ["remote get-url origin", "remote get-url --push origin", "config --get remote.origin.url"] {
        assert!(git_call_is_read(&argv(&format!("git {read}"))), "{read}");
        assert!(git_call_is_read(&argv(&format!("git -C /tmp/r1 {read}"))), "-C {read}");
    }
}

#[test]
fn each_config_reading_flag_and_scope_is_a_read() {
    for read in ["--get a.b", "--get-all a.b", "--get-regexp ^remote\\. url", "--list", "-l", "--list -z", "--null --list",
        "--includes --get a.b", "--global --get a.b", "--system --list", "--local --get a.b", "--worktree --list",
        "--file x --get key", "-f x --get key", "--file=x --list", "--blob HEAD:cfg --list", "--blob=HEAD:cfg --get a.b"]
    {
        assert!(git_call_is_read(&argv(&format!("git config {read}"))), "config {read}");
    }
}

#[test]
fn config_setting_and_unnamed_forms_are_writes() {
    for write in ["config user.name x", "config user.name", "config --unset a.b", "config --add a.b c",
        "config --replace-all a.b c", "config --rename-section a b", "config --remove-section a", "config --edit",
        "config -e", "config --get --unset a.b", "config --type=bool --get a.b", "config --show-origin --list",
        "config list", "config get a.b", "config", "config --file x", "config --global user.name x"]
    {
        assert!(!git_call_is_read(&argv(&format!("git {write}"))), "{write}");
    }
}

#[test]
fn a_config_file_value_is_not_taken_for_a_reading_flag() {
    assert!(!git_call_is_read(&argv("git config --file --list a.b c")));
    assert!(!git_call_is_read(&argv("git config --blob --get a.b c")));
}

#[test]
fn remote_listing_and_get_url_are_reads() {
    for read in ["remote", "remote -v", "remote --verbose", "remote get-url --all origin", "remote get-url origin --push",
        "remote get-url --push --all origin"]
    {
        assert!(git_call_is_read(&argv(&format!("git {read}"))), "{read}");
    }
}

#[test]
fn every_other_remote_form_is_a_write() {
    for write in ["remote add o u", "remote remove o", "remote rm o", "remote rename a b", "remote set-url o u",
        "remote set-head o -a", "remote set-branches o main", "remote prune o", "remote update", "remote show origin",
        "remote -v show", "remote get-url", "remote get-url a b", "remote get-url --bogus o", "remote -v -v"]
    {
        assert!(!git_call_is_read(&argv(&format!("git {write}"))), "{write}");
    }
}
