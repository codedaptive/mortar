use super::support::Fixture;
use gitio::GitError;
use join::JoinRunError;

#[test]
fn rit_a_another_type_comes_back_as_its_refusal_through_first_resolve() {
    // resolve -> write_conflicted_files -> classify_conflicted -> Git::tree_entry_typed -> From/Display.
    let f = Fixture::new();
    f.state(r#"["vendor/lib"]"#, r#"["vendor/lib"]"#, false);
    // Git's process seam can deliver another kind; the real gitio parser, conversion and formatter judge it.
    let response = format!("160000 tag {}\tvendor/lib\0", f.tip);
    let git = f.runner(response.as_bytes(), b"", Some(0));
    let before = f.snapshot();
    let JoinRunError::Git(error) = f.resolve(&git).unwrap_err() else { panic!("must refuse the non-gitlink") };
    assert_eq!(error, GitError { args: vec![], code: None, stderr: "not a blob: \"vendor/lib\" is a \"tag\"".into() });
    assert_eq!(f.trace(), "ls-tree\n");
    f.unchanged(&before);
}

#[test]
fn rit_a_a_git_failure_comes_back_unchanged_through_first_resolve() {
    // resolve -> classify_conflicted -> real gitio process failure/From; a failed read is not an absent/gitlink entry.
    let f = Fixture::new();
    f.state(r#"["vendor/lib"]"#, r#"["vendor/lib"]"#, false);
    let git = f.runner(b"", b"fatal: not a tree object\n", Some(128));
    let before = f.snapshot();
    let JoinRunError::Git(error) = f.resolve(&git).unwrap_err() else { panic!("must preserve the Git failure") };
    assert_eq!(error, GitError {
        args: ["ls-tree", "-z", "--full-tree", "--end-of-options", f.tree.as_str(), "--", "vendor/lib"]
            .map(str::to_string).to_vec(),
        code: Some(128), stderr: "fatal: not a tree object\n".into(),
    });
    assert_eq!(f.trace(), "ls-tree\n");
    f.unchanged(&before);
}

#[test]
fn rit_b_a_real_tree_comes_back_as_its_refusal_through_first_resolve() {
    // resolve -> classify_conflicted -> real ls-tree directory entry -> gitio From/Display.
    let f = Fixture::new();
    f.state(r#"["vendor/lib"]"#, r#"["vendor/lib"]"#, false);
    let git = f.runner(b"", b"", None);
    let before = f.snapshot();
    let JoinRunError::Git(error) = f.resolve(&git).unwrap_err() else { panic!("must refuse a folder, not list a gitlink") };
    assert_eq!(error, GitError { args: vec![], code: None, stderr: "not a blob: \"vendor/lib\" is a \"tree\"".into() });
    assert_eq!(f.trace(), "ls-tree\n");
    f.unchanged(&before);
}

#[test]
fn rit_b_a_commit_record_naming_another_path_is_refused_before_gitlink_classification() {
    // S-109 equivalent of the extracted mismatched-path NotABlob: the public reader checks paths first.
    // resolve -> tree_entry_typed/read_tree_entry_record -> classify_conflicted -> unchanged Git refusal.
    let f = Fixture::new();
    f.state(r#"["vendor/lib"]"#, r#"["vendor/lib"]"#, false);
    let response = format!("160000 commit {}\tvendor/other\0", f.tip);
    let git = f.runner(response.as_bytes(), b"", Some(0));
    let before = f.snapshot();
    let JoinRunError::Git(error) = f.resolve(&git).unwrap_err() else { panic!("must refuse another path, not list a gitlink") };
    assert_eq!(error, GitError {
        args: vec![], code: None,
        stderr: format!("not one ls-tree record for \"vendor/lib\" (another path): \"160000 commit {}\\tvendor/other\\0\"", f.tip),
    });
    assert_eq!(f.trace(), "ls-tree\n");
    f.unchanged(&before);
}
