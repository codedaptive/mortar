//! #1258/S-87/S-89 split: resolve -> persisted handoff tree -> real gitio index/file movement, no new dependency.
use super::*;

#[test]
fn two_round_resolution_returns_every_rounds_paths_over_old_tip_then_real_folder_move_keeps_unrelated_edits() {
    let Ready { r, caller_tree, folder_tree, to } = ready();
    let answer = r.resolve(&caller_tree).unwrap();
    assert_answer(answer, &r, &to, &folder_tree);
    assert_one_rebase(&r, &to);
    // S87's tree is NOT the final replay tree or the entire caller tree: it keeps the old tip's outside/one.
    assert_ne!(folder_tree, caller_tree);
    assert_ne!(folder_tree, r.f.git.commit_info(&to).unwrap().tree);
    assert_eq!(r.f.blob(&folder_tree, "outside"), b"old outside\n");
    assert_eq!(r.f.blob(&folder_tree, "one"), b"one\n");
    let log = r.log(); let refs = r.f.state().0;
    // Consume the handoff through the public gitio primitives already available to this crate's callers.
    // This proves tree semantics and real movement, not folders::move_folder's S88 fallback orchestration.
    let moved = caller(&r.f.git, &r.f, || {
        r.f.git.set_index(&r.f.repo.path, &folder_tree).map_err(JoinRunError::Git)?;
        assert_eq!(fs::read(r.f.repo.path.join("one")).unwrap(), b"unrelated local edit\n", "set_index changes no files");
        r.f.git.folder_move(&r.f.repo.path, &folder_tree, &to).map_err(JoinRunError::Git)
    }).unwrap();
    assert_eq!(moved, gitio::FolderMoveOutcome::Moved);
    assert_eq!(fs::read(r.f.repo.path.join(&r.path)).unwrap(), b"first resolution\n");
    assert_eq!(fs::read(r.f.repo.path.join("second")).unwrap(), b"final second resolution\n");
    assert_eq!(fs::read(r.f.repo.path.join("outside")).unwrap(), b"advanced zone work\n");
    assert_eq!(fs::read(r.f.repo.path.join("one")).unwrap(), b"unrelated local edit\n");
    assert_eq!(fs::read(r.f.repo.path.join("local-only")).unwrap(), b"local work must survive\n");
    assert_eq!(r.f.run(&["write-tree"]), format!("{}\n", r.f.git.commit_info(&to).unwrap().tree).as_bytes());
    assert_eq!(r.f.run(&["diff", "--name-only"]), b"one\n");
    assert_eq!(r.f.git.rev_parse("HEAD").unwrap(), Some(to));
    assert_eq!(r.f.state().0, refs); assert_eq!(r.log(), log, "caller folder movement adds no second rebase op");
}

#[test]
fn earlier_round_path_becoming_a_folder_refuses_the_final_handoff_and_keeps_the_pause() {
    let Ready { r, .. } = ready();
    fs::remove_file(r.f.repo.path.join(&r.path)).unwrap();
    fs::create_dir(r.f.repo.path.join(&r.path)).unwrap();
    fs::write(r.f.repo.path.join(&r.path).join("child"), b"earlier path is now a folder\n").unwrap();
    let caller_tree = r.f.tree(&[("conflict/child", b"earlier path is now a folder\n"), ("second", b"final second resolution\n"),
        ("outside", b"old outside\n"), ("one", b"unrelated local edit\n"), ("two", b"two\n")]);
    let before = r.f.state(); let state = r.state_text(); let log = r.log();
    assert!(state.contains("\"files\":[\"second\"],\"all_files\":[\"conflict\",\"second\"]"));
    assert!(matches!(r.resolve(&caller_tree), Err(JoinRunError::ConflictedPathIsFolder { path }) if path == PathBuf::from(&r.path)));
    r.f.assert_state(&before); assert_eq!(r.state_text(), state); assert_eq!(r.log(), log);
    assert_eq!(fs::read(r.f.repo.path.join(&r.path).join("child")).unwrap(), b"earlier path is now a folder\n");
    // The implementation may already have written an unreferenced resolved commit before building the handoff.
    // The contract here is refusal without a ref move or loss of the paused state, not zero object writes.
}

#[test]
fn killed_after_the_final_two_round_op_repeats_the_same_folder_tree_from_durable_state() {
    let Ready { r, caller_tree, folder_tree, to } = ready();
    super::crashes::kill_at_boundary(&r, &caller_tree, "after");
    assert_eq!(r.f.git.read_ref(REQUEST_REF).unwrap(), Some(to.clone()));
    let state = r.state_text();
    assert!(state.contains(&format!("\"rebased_to\":\"{to}\",\"folder_tree\":\"{folder_tree}\"")));
    assert!(state.contains("\"all_files\":[\"conflict\",\"second\"]"));
    // A recomputation from the new caller input would either fail or produce a different tree.
    fs::write(r.f.repo.path.join(&r.path), b"<<<<<<< later edit must not be reread\n").unwrap();
    let invalid_tree = Oid::parse(&"f".repeat(40)).unwrap();
    assert_answer(r.resolve(&invalid_tree).unwrap(), &r, &to, &folder_tree);
    assert_one_rebase(&r, &to);
    assert_eq!(fs::read(r.f.repo.path.join(&r.path)).unwrap(), b"<<<<<<< later edit must not be reread\n");
    assert!(matches!(r.resolve(&invalid_tree), Err(JoinRunError::NothingToResolve { .. })), "S89 post-removal recovery is not built by3d");
}

struct Ready { r: Paused, caller_tree: Oid, folder_tree: Oid, to: Oid }
fn ready() -> Ready {
    let r = Paused::new("conflict"); r.first();
    fs::write(r.f.repo.path.join(&r.path), b"first resolution\n").unwrap();
    let first_caller = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"old outside\n"), ("second", b"request second\n"), ("one", b"one\n"), ("two", b"two\n")]);
    let answer = r.resolve(&first_caller).unwrap();
    assert!(matches!(answer, JoinAnswer::Conflicted { commit, files, .. } if commit == r.extras[1] && files == vec![PathBuf::from("second")]));
    assert!(r.state_text().contains("\"files\":[\"second\"],\"all_files\":[\"conflict\",\"second\"]"));
    let answer = r.resolve(&first_caller).unwrap();
    assert!(matches!(answer, JoinAnswer::FilesWritten { files, .. } if files == vec![PathBuf::from("second")]));
    fs::write(r.f.repo.path.join("second"), b"final second resolution\n").unwrap();
    fs::write(r.f.repo.path.join("one"), b"unrelated local edit\n").unwrap();
    let caller_tree = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"old outside\n"), ("second", b"final second resolution\n"), ("one", b"unrelated local edit\n"), ("two", b"two\n")]);
    // Literal, independent tree oracle: old request tip except BOTH resolved paths, not just current second.
    let folder_tree = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"old outside\n"), ("second", b"final second resolution\n"), ("one", b"one\n"), ("two", b"two\n")]);
    let resolved_tree = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"advanced zone work\n"), ("second", b"zone second\n")]);
    let resolved = r.f.commit(&resolved_tree, &[r.zone.clone()], "conflicted message\n", &author("Zoë", "1700000100 -0530"));
    let one_tree = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"advanced zone work\n"), ("second", b"zone second\n"), ("one", b"one\n")]);
    let one = r.f.commit(&one_tree, &[resolved], "one message\n", &author("One", "1700000200 +0000"));
    let final_tree = r.f.tree(&[(&r.path, b"first resolution\n"), ("outside", b"advanced zone work\n"), ("second", b"final second resolution\n"), ("one", b"one\n"), ("two", b"two\n")]);
    let to = r.f.commit(&final_tree, &[one], "two message\n", &author("Two", "1700000300 +0000"));
    Ready { r, caller_tree, folder_tree, to }
}
fn assert_answer(answer: JoinAnswer, r: &Paused, to: &Oid, folder_tree: &Oid) {
    let JoinAnswer::Rebased { request_ref, from, to: actual, folder_tree: actual_tree } = answer else { panic!("expected Rebased") };
    assert_eq!(request_ref, REQUEST_REF); assert_eq!(from, r.tip);
    assert_eq!(&actual, to); assert_eq!(actual_tree, Some(folder_tree.clone()));
}
