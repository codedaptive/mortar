use super::*;

#[test]
fn real_state_encoding_and_decoding_round_trip_escaped_unicode_and_control_paths_through_resolve() {
    // Port of the extracted codec roundtrip through PUBLIC join writes/resolve reads, for reachable real states.
    let controls: String = (1u32..0x20).map(|c| char::from_u32(c).unwrap()).collect();
    for name in ["a\"b\\c\nd\te\rf\u{8}g\u{c}h".to_string(), controls, "café/日本.txt".into(), "x,y]z".into()] {
        let r = Paused::named(&name, "none");
        assert!(r.state_text().contains("\"files_written\":false"));
        r.first(); // The decoded name is used to write the actual file and is returned byte-for-byte.
        assert!(!r.f.repo.path.join(".git/mortar-join.json.tmp").exists());
        fs::write(r.f.repo.path.join(&name), b"resolved\n").unwrap();
        let to = rebased(r.resolve(&r.caller_tree()).unwrap(), &r.tip);
        let info = r.f.git.commit_info(&to).unwrap();
        assert_eq!(r.f.blob(&info.tree, &name), b"resolved\n");
        assert_one_rebase(&r, &to);
    }
}

#[test]
fn rename_rename_absent_original_is_skipped_then_deleted_without_reverting_unrelated_zone_work() {
    let f = Fixture::new();
    let payload: &[u8] = b"identical long original rename payload\n";
    let base_tree = f.tree(&[("original", payload), ("outside", b"old outside\n")]);
    let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
    let zone_tree = f.tree(&[("ours", payload), ("outside", b"advanced zone work\n")]);
    let zone = f.commit(&zone_tree, std::slice::from_ref(&base), "zone rename\n", &author("Zone", "1700000100 +0000"));
    let request_tree = f.tree(&[("theirs", payload), ("outside", b"old outside\n")]);
    let tip = f.commit(&request_tree, std::slice::from_ref(&base), "request rename\n", &author("Topic", "1700000200 +0000"));
    join_refs(&f, &zone, &tip);
    let r = Paused { f, path: "original".into(), base, bad: tip.clone(), tip, zone, extras: vec![] };
    let answer = caller(&r.f.git, &r.f, || join::join(&r.f.git, &r.f.repo.path, JOIN_ZONE, REQUEST_REF, ACTOR)).unwrap();
    assert!(matches!(answer, JoinAnswer::Conflicted { files, .. } if files.contains(&PathBuf::from("original"))));
    let caller_tree = r.f.tree(&[("ours", b"chosen rename resolution\n"), ("outside", b"stale caller outside\n")]);
    let first = r.resolve(&caller_tree).unwrap();
    assert!(matches!(first, JoinAnswer::FilesWritten { files, skipped, gitlinks, .. }
        if skipped == vec![PathBuf::from("original")] && files.contains(&PathBuf::from("ours")) && files.contains(&PathBuf::from("theirs")) && gitlinks.is_empty()));
    assert!(!r.f.repo.path.join("original").exists());
    fs::write(r.f.repo.path.join("ours"), b"chosen rename resolution\n").unwrap();
    fs::remove_file(r.f.repo.path.join("theirs")).unwrap();
    let to = rebased(r.resolve(&caller_tree).unwrap(), &r.tip);
    assert_eq!(r.f.git.commit_info(&to).unwrap().tree, r.f.tree(&[("ours", b"chosen rename resolution\n"), ("outside", b"advanced zone work\n")]));
    assert_one_rebase(&r, &to);
}

#[test]
fn conflicted_gitlink_is_listed_not_written_and_directory_markers_are_skipped_for_caller_tree_overlay() {
    let f = Fixture::new(); let empty = f.tree(&[]);
    let sub_base = f.commit(&empty, &[], "sub base\n", &author("Sub", "1700000000 +0000"));
    let sub_zone = f.commit(&empty, std::slice::from_ref(&sub_base), "sub zone\n", &author("Sub", "1700000100 +0000"));
    let sub_request = f.commit(&empty, std::slice::from_ref(&sub_base), "sub request\n", &author("Sub", "1700000200 +0000"));
    let sub_choice = f.commit(&empty, &[sub_zone.clone(), sub_request.clone()], "sub chosen\n", &author("Sub", "1700000300 +0000"));
    let base_tree = gitlink_tree(&f, &sub_base, b"old outside\n");
    let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
    let zone_tree = gitlink_tree(&f, &sub_zone, b"advanced zone work\n");
    let zone = f.commit(&zone_tree, std::slice::from_ref(&base), "zone submodule\n", &author("Zone", "1700000100 +0000"));
    let request_tree = gitlink_tree(&f, &sub_request, b"old outside\n");
    let tip = f.commit(&request_tree, std::slice::from_ref(&base), "request submodule\n", &author("Topic", "1700000200 +0000"));
    join_refs(&f, &zone, &tip);
    let r = Paused { f, path: "module".into(), base, bad: tip.clone(), tip, zone, extras: vec![] };
    let answer = caller(&r.f.git, &r.f, || join::join(&r.f.git, &r.f.repo.path, JOIN_ZONE, REQUEST_REF, ACTOR)).unwrap();
    assert!(matches!(answer, JoinAnswer::Conflicted { files, .. } if files == vec![PathBuf::from("module")]));
    fs::create_dir_all(r.f.repo.path.join("module")).unwrap();
    fs::write(r.f.repo.path.join("module/keep"), b"<<<<<<< nested submodule bytes stay\n").unwrap();
    let caller_tree = gitlink_tree(&r.f, &sub_choice, b"stale caller outside\n");
    let before = r.f.state();
    let answer = r.resolve(&caller_tree).unwrap();
    assert!(matches!(answer, JoinAnswer::FilesWritten { files, skipped, gitlinks, .. }
        if files.is_empty() && skipped.is_empty() && gitlinks == vec![PathBuf::from("module")]));
    r.f.assert_state(&before); assert!(r.log().is_empty());
    assert_eq!(fs::read(r.f.repo.path.join("module/keep")).unwrap(), b"<<<<<<< nested submodule bytes stay\n");
    let to = rebased(r.resolve(&caller_tree).unwrap(), &r.tip);
    let info = r.f.git.commit_info(&to).unwrap();
    assert_eq!(r.f.run(&["ls-tree", info.tree.as_str(), "--", "module"]), format!("160000 commit {sub_choice}\tmodule\n").as_bytes());
    assert_eq!(r.f.blob(&info.tree, "outside"), b"advanced zone work\n");
    assert_eq!(fs::read(r.f.repo.path.join("module/keep")).unwrap(), b"<<<<<<< nested submodule bytes stay\n");
    assert_one_rebase(&r, &to);
}

#[test]
fn a_non_gitlink_git_error_is_not_swallowed_by_first_write_classification() {
    let r = Paused::new("none");
    let shim = r.f.repo.path.join(".git/refuse-tree.sh");
    testkit::write_git_shim(&shim, b"#!/bin/sh\nif [ \"${3-}\" = ls-tree ]; then printf 'fixture tree read failed\\n' >&2; exit 67; fi\nexec git \"$@\"\n").unwrap();
    let git = Git { program: shim, ..r.f.git.clone() };
    let tree = r.caller_tree(); let before = r.f.state(); let text = r.state_text();
    assert!(matches!(r.resolve_with(&git, &tree), Err(JoinRunError::Git(error)) if error.code == Some(67) && error.stderr.contains("fixture tree read failed")));
    r.f.assert_state(&before); assert_eq!(r.state_text(), text); assert!(r.log().is_empty());
}

fn gitlink_tree(f: &Fixture, sub: &Oid, outside: &[u8]) -> Oid {
    f.tree(&[("outside", outside), (".gitmodules", b"[submodule \"module\"]\n\tpath = module\n\turl = ./unused\n")]);
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), f.repo.path.join(".git/oracle-index").into_os_string())], ..RunOpts::default() };
    f.git.run(&["update-index", "--add", "--cacheinfo", "160000", sub.as_str(), "module"], &opts).unwrap();
    oid(&f.git.run(&["write-tree"], &opts).unwrap().stdout)
}
