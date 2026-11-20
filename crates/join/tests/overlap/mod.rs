//! Joins join::overlap -> gitio::Git::changed_files -> real Git commit trees.
//! Reuses this integration target's private real-repository fixture; no production source inclusion.
use super::{author, Fixture};
use gitio::Oid;
use join::{overlap, JoinError, OpenRequest, OverlapPair};
use std::{fs, path::{Path, PathBuf}};

#[test]
fn every_same_path_pair_names_the_requests_and_exact_paths_independent_of_input_order() {
    // Public overlap -> changed_files: three real forks share two paths, not their private additions.
    let f = Fixture::new();
    let base = commit(&f, &[("a/b", b"base\n"), ("a-b", b"base\n")], &[], "base\n");
    let alpha = commit(&f, &[("a/b", b"alpha\n"), ("a-b", b"alpha\n"), ("alpha-only", b"one\n")], std::slice::from_ref(&base), "alpha\n");
    let middle = commit(&f, &[("a/b", b"middle\n"), ("a-b", b"middle\n"), ("middle-only", b"two\n")], std::slice::from_ref(&base), "middle\n");
    let zeta = commit(&f, &[("a/b", b"zeta\n"), ("a-b", b"zeta\n"), ("zeta-only", b"three\n")], std::slice::from_ref(&base), "zeta\n");
    f.anchor(&base, &alpha);
    let mut requests = vec![request("zeta", &base, &zeta), request("alpha", &base, &alpha), request("middle", &base, &middle)];
    // Literals come from the fixture edits, not changed_files or the overlap answer.
    let expected = vec![
        pair("alpha", "middle", &["a/b", "a-b"]),
        pair("alpha", "zeta", &["a/b", "a-b"]),
        pair("middle", "zeta", &["a/b", "a-b"]),
    ];
    let before = observe(&f);
    assert_eq!(overlap(&f.git, &requests).unwrap(), expected);
    requests.reverse();
    assert_eq!(overlap(&f.git, &requests).unwrap(), expected);
    assert_eq!(observe(&f), before);
}

#[test]
fn disjoint_requests_use_each_callers_base_and_empty_or_unchanged_requests_add_no_pair() {
    // Public overlap -> real per-request ranges; using the first base for both would falsely share left.
    let f = Fixture::new();
    let base_a = commit(&f, &[("left", b"zero\n"), ("right", b"zero\n")], &[], "base a\n");
    let base_b = commit(&f, &[("left", b"upstream\n"), ("right", b"zero\n")], std::slice::from_ref(&base_a), "base b\n");
    let left = commit(&f, &[("left", b"request\n"), ("right", b"zero\n")], std::slice::from_ref(&base_a), "left\n");
    let right = commit(&f, &[("left", b"upstream\n"), ("right", b"request\n")], std::slice::from_ref(&base_b), "right\n");
    f.anchor(&base_b, &left);
    let before = observe(&f);
    assert!(overlap(&f.git, &[]).unwrap().is_empty());
    assert!(overlap(&f.git, &[request("left", &base_a, &left)]).unwrap().is_empty());
    assert!(overlap(&f.git, &[
        request("left", &base_a, &left), request("right", &base_b, &right),
        request("unchanged", &base_b, &base_b),
    ]).unwrap().is_empty());
    assert_eq!(observe(&f), before);
}

#[test]
fn supplied_snapshot_commit_finds_unsaved_overlap_while_request_branch_tip_does_not() {
    // Caller gitio::snapshot_tree -> real snapshot commit -> overlap; join must not rediscover a branch tip.
    let f = Fixture::new();
    let base = commit(&f, &[("shared", b"base\n")], &[], "base\n");
    let other = commit(&f, &[("shared", b"other request\n")], std::slice::from_ref(&base), "other\n");
    f.anchor(&base, &base); // Both actual branch tips deliberately precede the unsaved edit.
    fs::write(f.repo.path.join("shared"), b"unsaved snapshot bytes\n").unwrap();
    let tree = f.git.snapshot_tree(&base, &[]).unwrap();
    let snapshot = f.commit(&tree, std::slice::from_ref(&base), "caller snapshot\n", &author("Snapshot", "1700000100 +0000"));
    assert_eq!(f.blob(&snapshot, "shared"), b"unsaved snapshot bytes\n");
    assert_eq!(f.run(&["rev-parse", "refs/heads/topic"]), format!("{base}\n").as_bytes());
    let before = observe(&f);
    assert!(overlap(&f.git, &[request("unsaved", &base, &base), request("other", &base, &other)]).unwrap().is_empty());
    assert_eq!(overlap(&f.git, &[
        request("unsaved", &base, &snapshot), request("other", &base, &other),
    ]).unwrap(), vec![pair("other", "unsaved", &["shared"])]);
    assert_eq!(observe(&f), before);
}

#[test]
fn a_real_rename_overlaps_both_its_old_and_new_path_including_spaces_and_newlines() {
    // Public overlap -> changed_files: a real R100 rename must participate at both ends, without path splitting.
    let f = Fixture::new();
    let old = "old\nname";
    let new = "new name";
    let base = commit(&f, &[(old, b"rename payload\n")], &[], "base\n");
    let renamed = commit(&f, &[(new, b"rename payload\n")], std::slice::from_ref(&base), "rename\n");
    let edit_old = commit(&f, &[(old, b"edited original\n")], std::slice::from_ref(&base), "old user\n");
    let add_new = commit(&f, &[(old, b"rename payload\n"), (new, b"added independently\n")], std::slice::from_ref(&base), "new user\n");
    f.anchor(&base, &renamed);
    assert_eq!(f.run(&["diff", "--name-status", "-z", "-M", base.as_str(), renamed.as_str()]), b"R100\0old\nname\0new name\0");
    let before = observe(&f);
    assert_eq!(overlap(&f.git, &[
        request("rename", &base, &renamed), request("old-user", &base, &edit_old), request("new-user", &base, &add_new),
    ]).unwrap(), vec![pair("new-user", "rename", &["new name"]), pair("old-user", "rename", &["old\nname"])]);
    assert_eq!(observe(&f), before);
}

#[test]
fn overlap_and_repeat_preserve_all_refs_dirty_index_work_files_and_repository_files() {
    // Public overlap -> read-only Git diff; exact recursive bytes also catch created objects, logs or state files.
    let f = Fixture::new();
    let base = commit(&f, &[("shared", b"base\n")], &[], "base\n");
    let left = commit(&f, &[("shared", b"left\n")], std::slice::from_ref(&base), "left\n");
    let right = commit(&f, &[("shared", b"right\n")], std::slice::from_ref(&base), "right\n");
    f.anchor(&base, &left);
    fs::write(f.repo.path.join("shared"), b"staged bytes\n").unwrap();
    f.run(&["add", "--", "shared"]);
    fs::write(f.repo.path.join("shared"), b"unstaged bytes\n").unwrap();
    fs::create_dir(f.repo.path.join("untracked")).unwrap();
    fs::write(f.repo.path.join("untracked/local\nfile"), b"untracked bytes\0\xff").unwrap();
    assert_eq!(f.run(&["show", ":shared"]), b"staged bytes\n");
    let before = observe(&f);
    let requests = [request("left", &base, &left), request("right", &base, &right)];
    for _ in 0..2 {
        assert_eq!(overlap(&f.git, &requests).unwrap(), vec![pair("left", "right", &["shared"])]);
        assert_eq!(observe(&f), before);
    }
    assert_eq!(fs::read(f.repo.path.join("shared")).unwrap(), b"unstaged bytes\n");
}

#[test]
fn missing_base_and_missing_tip_are_typed_git_refusals_not_partial_overlap_answers() {
    // Public overlap -> changed_files -> real failing git diff, with exact failed range and no mutation.
    let f = Fixture::new();
    let base = commit(&f, &[("shared", b"base\n")], &[], "base\n");
    let left = commit(&f, &[("shared", b"left\n")], std::slice::from_ref(&base), "left\n");
    let right = commit(&f, &[("shared", b"right\n")], std::slice::from_ref(&base), "right\n");
    f.anchor(&base, &left);
    let missing = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    assert!(f.git.run(&["cat-file", "-e", missing.as_str()], &gitio::RunOpts::default()).unwrap_err().code.is_some_and(|code| code != 0));
    let before = observe(&f);
    for (bad_base, bad_tip) in [(&missing, &right), (&base, &missing)] {
        let error = overlap(&f.git, &[
            request("left", &base, &left), request("right", &base, &right), request("bad", bad_base, bad_tip),
        ]).unwrap_err();
        let JoinError::Git(error) = error else { panic!("expected typed Git refusal: {error:?}") };
        assert_eq!(error.code, Some(128));
        assert_eq!(error.args, ["diff", "--name-only", "-z", "--no-renames", bad_base.as_str(), bad_tip.as_str()]);
        assert!(error.stderr.contains(missing.as_str()), "{error:?}");
        assert_eq!(observe(&f), before);
    }
}

fn request(name: &str, base: &Oid, tip: &Oid) -> OpenRequest {
    OpenRequest { name: name.into(), base: base.clone(), tip: tip.clone() }
}

fn pair(first: &str, second: &str, paths: &[&str]) -> OverlapPair {
    OverlapPair { first: first.into(), second: second.into(), paths: paths.iter().map(PathBuf::from).collect() }
}

fn commit(f: &Fixture, files: &[(&str, &[u8])], parents: &[Oid], message: &str) -> Oid {
    f.commit(&f.tree(files), parents, message, &author("Overlap fixture", "1700000000 +0000"))
}

#[derive(Debug, PartialEq, Eq)]
struct Observed {
    refs_index_head: (Vec<u8>, Vec<u8>, Vec<u8>),
    // Directories have None; files have exact bytes. Include .git, so additions/deletions cannot hide.
    entries: Vec<(PathBuf, Option<Vec<u8>>)>,
}

fn observe(f: &Fixture) -> Observed {
    let mut entries = Vec::new();
    read_entries(&f.repo.path, &f.repo.path, &mut entries);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Observed { refs_index_head: f.state(), entries }
}

fn read_entries(root: &Path, dir: &Path, entries: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            entries.push((relative, None));
            read_entries(root, &path, entries);
        } else {
            assert!(kind.is_file(), "fixture contains unexpected non-file: {}", path.display());
            entries.push((relative, Some(fs::read(&path).unwrap())));
        }
    }
}
