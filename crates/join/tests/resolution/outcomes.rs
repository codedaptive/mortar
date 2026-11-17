use super::*;

#[test]
fn missing_corrupt_and_unreadable_state_refuse_without_moving_history() {
    for kind in ["missing", "corrupt", "directory"] {
        let r = Paused::new("none"); let tree = r.caller_tree();
        let path = r.state_path();
        match kind { "missing" => fs::remove_file(&path).unwrap(), "corrupt" => fs::write(&path, b"not a state\n").unwrap(), _ => { fs::remove_file(&path).unwrap(); fs::create_dir(&path).unwrap(); } }
        let before = r.f.state(); let commits = r.commits();
        match (kind, r.resolve(&tree).unwrap_err()) {
            ("missing", JoinRunError::NothingToResolve { state }) => assert_eq!(state, path),
            ("corrupt", JoinRunError::StateCorrupt { path: actual, reason }) => { assert_eq!(actual, path); assert!(!reason.is_empty()); },
            ("directory", JoinRunError::StateRead { path: actual, .. }) => assert_eq!(actual, path),
            (_, other) => panic!("{kind}: {other:?}"),
        }
        r.f.assert_state(&before); assert_eq!(r.commits(), commits); assert!(r.log().is_empty());
    }
}

#[test]
fn first_write_repeat_markers_then_clean_resolve_commits_only_conflicted_paths_and_one_op() {
    // Ports continue_replay's overlay/no-remaining case through public resolve and actual objects.
    let r = Paused::new("none"); let original_state = r.state_text();
    let before_refs = r.f.state().0; let before_head = r.f.state().2;
    r.first();
    let marked = fs::read(r.f.repo.path.join(&r.path)).unwrap();
    assert_eq!(r.f.state().0, before_refs); assert_eq!(r.f.state().2, before_head); assert!(r.log().is_empty());
    // The persisted state from before files_written was saved models an interruption after files were written.
    fs::write(r.state_path(), original_state).unwrap();
    fs::write(r.f.repo.path.join(&r.path), b"partial edit\n").unwrap();
    r.first(); assert_eq!(fs::read(r.f.repo.path.join(&r.path)).unwrap(), marked);
    fs::write(r.f.repo.path.join(&r.path), b"line\n<<<<<<< ours\nx\n=======\ny\n>>>>>>> theirs\n").unwrap();
    let tree = r.caller_tree(); let before = r.f.state(); let state = r.state_text(); let commits = r.commits();
    let JoinRunError::StillMarked(files) = r.resolve(&tree).unwrap_err() else { panic!("markers must refuse") };
    assert_eq!(files, vec![join::MarkedFile { path: PathBuf::from(&r.path), lines: vec![2, 4, 6] }]);
    r.f.assert_state(&before); assert_eq!(r.state_text(), state); assert_eq!(r.commits(), commits); assert!(r.log().is_empty());
    fs::write(r.f.repo.path.join(&r.path), b"resolved\n").unwrap();
    fs::write(r.f.repo.path.join("outside"), b"unrelated dirty local bytes\n").unwrap();
    let to = rebased(r.resolve(&tree).unwrap(), &r.tip);
    let info = r.f.git.commit_info(&to).unwrap();
    assert_eq!(info.parents, vec![r.zone.clone()]); assert_eq!(info.message, "conflicted message\n");
    assert_eq!(info.author, author("Zoë", "1700000100 -0530"));
    assert_eq!(info.tree, r.f.tree(&[(&r.path, b"resolved\n"), ("outside", b"advanced zone work\n"), ("second", b"base second\n")]));
    assert_eq!(fs::read(r.f.repo.path.join("outside")).unwrap(), b"unrelated dirty local bytes\n");
    assert_eq!(r.f.state().1, before.1, "resolve moves history, not the caller's index to the new tree");
    assert_one_rebase(&r, &to);
    let log = r.log();
    assert!(matches!(r.resolve(&tree), Err(JoinRunError::NothingToResolve { .. })));
    assert_eq!(r.log(), log);
}

#[test]
fn two_remaining_commits_chain_from_the_resolved_start_not_the_original_onto() {
    // Ports both continue_replay's clean chain and replay_commits' start != onto composition.
    let r = Paused::new("clean"); r.first();
    fs::write(r.f.repo.path.join(&r.path), b"resolved\n").unwrap();
    let to = rebased(r.resolve(&r.caller_tree()).unwrap(), &r.tip);
    let two = r.f.git.commit_info(&to).unwrap();
    let one = r.f.git.commit_info(&two.parents[0]).unwrap();
    let resolved = r.f.git.commit_info(&one.parents[0]).unwrap();
    assert_eq!(two.message, "two message\n"); assert_eq!(one.message, "one message\n");
    assert_eq!(resolved.message, "conflicted message\n"); assert_eq!(resolved.parents, vec![r.zone.clone()]);
    assert_ne!(one.parents[0], r.zone);
    assert_eq!(two.tree, r.f.tree(&[(&r.path, b"resolved\n"), ("outside", b"advanced zone work\n"), ("second", b"base second\n"), ("one", b"one\n"), ("two", b"two\n")]));
    assert_eq!(one.author, author("One", "1700000200 +0000")); assert_eq!(two.author, author("Two", "1700000300 +0000"));
    assert_one_rebase(&r, &to);
}

#[test]
fn a_later_conflict_saves_new_pause_and_continuing_it_preserves_the_resolved_start() {
    let r = Paused::new("conflict"); r.first();
    fs::write(r.f.repo.path.join(&r.path), b"resolved\n").unwrap();
    let before_refs = r.f.state().0;
    let answer = r.resolve(&r.caller_tree()).unwrap();
    assert!(matches!(answer, JoinAnswer::Conflicted { commit, files, .. } if commit == r.extras[1] && files == vec![PathBuf::from("second")]));
    assert_eq!(r.f.state().0, before_refs); assert!(r.log().is_empty());
    let text = r.state_text();
    assert!(text.contains("\"files_written\":false,\"rebased_to\":null"));
    assert!(text.contains("\"remaining\":[]"));
    let answer = r.resolve(&r.caller_tree()).unwrap();
    assert!(matches!(answer, JoinAnswer::FilesWritten { commit, files, .. } if commit == r.extras[1] && files == vec![PathBuf::from("second")]));
    fs::write(r.f.repo.path.join("second"), b"second resolved\n").unwrap();
    let caller_tree = r.f.tree(&[("second", b"second resolved\n"), ("outside", b"caller stale\n")]);
    let to = rebased(r.resolve(&caller_tree).unwrap(), &r.tip);
    let info = r.f.git.commit_info(&to).unwrap();
    assert_eq!(r.f.blob(&info.tree, &r.path), b"resolved\n");
    assert_eq!(r.f.blob(&info.tree, "one"), b"one\n"); assert_eq!(r.f.blob(&info.tree, "two"), b"two\n");
    assert_eq!(r.f.blob(&info.tree, "outside"), b"advanced zone work\n"); assert_eq!(r.f.blob(&info.tree, "second"), b"second resolved\n");
    assert_one_rebase(&r, &to);
}

#[test]
fn first_replay_conflict_stops_at_onto_with_every_later_commit_remaining_and_no_commit_write() {
    // Ports nonempty replay_commits' first-conflict composition through public replay with real Git.
    let r = Paused::new("clean");
    let (git, trace) = r.f.runner(true).unwrap();
    let result = join::replay(&git, &r.base, REQUEST_REF, &r.zone).unwrap();
    assert!(matches!(result, join::ReplayResult::Conflicted { commit, replayed_tip, remaining, .. }
        if commit == r.bad && replayed_tip == r.zone && remaining == r.extras));
    let calls = fs::read_to_string(trace).unwrap();
    assert_eq!(calls.lines().filter(|line| *line == "merge-tree").count(), 1);
    assert!(!calls.lines().any(|line| line == "commit-tree"));
    assert_eq!(r.f.git.read_ref(REQUEST_REF).unwrap(), Some(r.tip.clone())); assert!(r.log().is_empty());
}

#[test]
fn real_directory_marker_skip_still_refuses_a_folder_in_the_callers_tree_before_committing() {
    let r = Paused::new("none"); r.first();
    fs::remove_file(r.f.repo.path.join(&r.path)).unwrap(); fs::create_dir(r.f.repo.path.join(&r.path)).unwrap();
    fs::write(r.f.repo.path.join(&r.path).join("child"), b"<<<<<<< a nested file must not be read\n").unwrap();
    let caller_tree = r.f.tree(&[("conflict/child", b"a folder entry\n")]);
    let before = r.f.state(); let text = r.state_text(); let commits = r.commits();
    assert!(matches!(r.resolve(&caller_tree), Err(JoinRunError::ConflictedPathIsFolder { path }) if path == std::path::Path::new(&r.path)));
    r.f.assert_state(&before); assert_eq!(r.state_text(), text); assert_eq!(r.commits(), commits); assert!(r.log().is_empty());
    assert_eq!(fs::read(r.f.repo.path.join(&r.path).join("child")).unwrap(), b"<<<<<<< a nested file must not be read\n");
}

#[test]
fn missing_onto_is_named_before_overlay_or_commit_even_with_an_invalid_caller_tree() {
    let r = Paused::new("clean"); r.first(); fs::write(r.f.repo.path.join(&r.path), b"resolved\n").unwrap();
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    r.replace_state(&format!("\"onto\":\"{}\"", r.zone), &format!("\"onto\":\"{missing}\""));
    let before = r.f.state(); let text = r.state_text(); let commits = r.commits();
    let invalid_tree = Oid::parse(&"f".repeat(40)).unwrap();
    assert!(matches!(r.resolve(&invalid_tree), Err(JoinRunError::Replay(JoinError::OntoMissing(id))) if id == missing));
    r.f.assert_state(&before); assert_eq!(r.state_text(), text); assert_eq!(r.commits(), commits); assert!(r.log().is_empty());
}

#[test]
fn changed_or_deleted_request_ref_is_stale_with_state_and_all_local_work_kept() {
    for deleted in [false, true] {
        let r = Paused::new("none"); let tree = r.caller_tree();
        if deleted { r.f.run(&["update-ref", "-d", REQUEST_REF]); r.f.run(&["symbolic-ref", "HEAD", "refs/heads/main"]); }
        else { r.f.run(&["update-ref", REQUEST_REF, r.zone.as_str()]); }
        let before = r.f.state(); let text = r.state_text(); let bytes = fs::read(r.f.repo.path.join(&r.path)).unwrap();
        let error = r.resolve(&tree).unwrap_err();
        assert!(matches!(error, JoinRunError::StateStale { request_ref, expected, actual, state }
            if request_ref == REQUEST_REF && expected == r.tip && actual == if deleted { None } else { Some(r.zone.clone()) } && state == r.state_path()));
        r.f.assert_state(&before); assert_eq!(r.state_text(), text); assert_eq!(fs::read(r.f.repo.path.join(&r.path)).unwrap(), bytes); assert!(r.log().is_empty());
    }
}
