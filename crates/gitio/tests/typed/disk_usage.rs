//! #1179/S-75: disk_usage_only joins real Git walks, refs and linked HEADs; loose sizes are independent bounds.
use crate::fixture::{commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{Git, Oid, RunOpts};
use std::fs;

const SELECTED: &str = "refs/mortar/snap/red/closed";

#[test]
fn unique_objects_count_once_within_independent_loose_object_bounds_and_no_repo_mutation() {
    let r = repo();
    let base_tree = tree(&r.git, &[("base", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    run(&r.git, &["checkout", "--force", "main"]);
    let selected_tree = tree(&r.git, &[("unique", b"unique payload with no other owner\n")]);
    let selected = commit(&r.git, &selected_tree, &[base], "selected\n");
    set_ref(&r.git, SELECTED, &selected);
    let blob = r.git.tree_entry(&selected_tree, "unique".as_ref()).unwrap().unwrap();
    let min = loose_size(&r, &blob);
    let max = min + loose_size(&r, &selected_tree) + loose_size(&r, &selected);
    fs::write(r.path.join("base"), b"dirty staged\n").unwrap();
    run(&r.git, &["add", "--", "base"]);
    fs::write(r.path.join("base"), b"dirty unstaged\n").unwrap();
    fs::write(r.path.join("untracked"), b"untouched\0\xff").unwrap();
    let before = observe(&r);
    let bytes = r.git.disk_usage_only(&[SELECTED]).unwrap();
    assert!(min > 0 && (min..=max).contains(&bytes), "unique bytes {bytes}, independent {min}..={max}");
    assert_eq!(r.git.disk_usage_only(&[SELECTED, SELECTED]).unwrap(), bytes, "duplicate names must not double count");
    assert_eq!(observe(&r), before);
}

#[test]
fn a_blob_shared_with_another_tip_is_not_counted() {
    let r = repo();
    let contents = tree(&r.git, &[("shared", b"the same blob is held by another ref\n")]);
    let common = commit(&r.git, &contents, &[], "shared ancestor\n");
    let selected = commit(&r.git, &contents, &[common.clone()], "selected child\n");
    let kept = commit(&r.git, &contents, &[common], "kept child\n");
    set_ref(&r.git, SELECTED, &selected);
    set_ref(&r.git, "refs/heads/main", &kept);
    // The other tip directly holds the same tree, so only the unique selected commit can count here.
    assert_eq!(r.git.disk_usage_only(&[SELECTED]).unwrap(), loose_size(&r, &selected));
}

#[test]
fn checked_out_ref_and_ref_contained_by_detached_head_count_zero() {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"held by HEAD\n")]);
    let selected = commit(&r.git, &contents, &[], "selected\n");
    set_ref(&r.git, SELECTED, &selected);
    run(&r.git, &["checkout", "--detach", selected.as_str()]);
    assert_eq!(r.git.disk_usage_only(&[SELECTED]).unwrap(), 0);
    let descendant = commit(&r.git, &contents, &[selected.clone()], "descendant\n");
    run(&r.git, &["checkout", "--detach", descendant.as_str()]);
    assert_eq!(r.git.disk_usage_only(&[SELECTED]).unwrap(), 0, "detached HEAD keeps its ancestors alive too");
    set_ref(&r.git, "refs/heads/checked", &selected);
    run(&r.git, &["checkout", "checked"]);
    assert_eq!(r.git.disk_usage_only(&["refs/heads/checked"]).unwrap(), 0);
}

#[test]
fn a_linked_worktrees_detached_head_and_its_ancestors_are_other_roots() {
    let r = repo();
    let empty = tree(&r.git, &[]);
    let base = commit(&r.git, &empty, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    let contents = tree(&r.git, &[("file", b"only linked HEAD holds this\n")]);
    let selected = commit(&r.git, &contents, &[base], "selected\n");
    set_ref(&r.git, SELECTED, &selected);
    assert!(r.git.disk_usage_only(&[SELECTED]).unwrap() > 0);
    let linked = r.path.join("linked");
    run(&r.git, &["worktree", "add", "--detach", linked.to_str().unwrap(), selected.as_str()]);
    assert_eq!(r.git.disk_usage_only(&[SELECTED]).unwrap(), 0);
    let descendant = commit(&r.git, &contents, &[selected], "linked descendant\n");
    let local = Git { repo: linked, ..r.git.clone() };
    run(&local, &["checkout", "--detach", descendant.as_str()]);
    assert_eq!(r.git.disk_usage_only(&[SELECTED]).unwrap(), 0);
}

#[test]
fn older_shared_blob_and_parentless_shared_tree_obey_upper_bound_not_exact_reclamation() {
    let r = repo();
    let old_tree = tree(&r.git, &[("old", b"payload once held in old history\n")]);
    let old = commit(&r.git, &old_tree, &[], "old\n");
    let current_tree = tree(&r.git, &[("current", b"current kept files\n")]);
    let kept = commit(&r.git, &current_tree, &[old.clone()], "kept descendant\n");
    set_ref(&r.git, "refs/heads/main", &kept);
    let selected = commit(&r.git, &old_tree, &[], "parentless reuse\n");
    set_ref(&r.git, SELECTED, &selected);
    let blob = r.git.tree_entry(&old_tree, "old".as_ref()).unwrap().unwrap();
    let min = loose_size(&r, &selected);
    let max = min + loose_size(&r, &old_tree) + loose_size(&r, &blob);
    let bytes = r.git.disk_usage_only(&[SELECTED]).unwrap();
    assert!((min..=max).contains(&bytes), "S75 permits historical sharing overcount: {bytes}, {min}..={max}");
    // Re-adding the old blob atop the current kept tip is the other documented overcount shape.
    let readded_tree = tree(&r.git, &[("current", b"current kept files\n"), ("old", b"payload once held in old history\n")]);
    let readded = commit(&r.git, &readded_tree, &[kept], "re-add old\n");
    set_ref(&r.git, SELECTED, &readded);
    let min = loose_size(&r, &readded) + loose_size(&r, &readded_tree);
    let bytes = r.git.disk_usage_only(&[SELECTED]).unwrap();
    assert!((min..=min + loose_size(&r, &blob)).contains(&bytes), "readded upper bound: {bytes}");
}

#[test]
fn unknown_ref_names_it_and_empty_or_invalid_selection_never_spawns_git() {
    let r = repo();
    let error = r.git.disk_usage_only(&["refs/mortar/snap/red/unknown"]).unwrap_err();
    assert_eq!(error.code, None);
    assert!(error.stderr.contains("unknown ref") && error.stderr.contains("refs/mortar/snap/red/unknown"));
    let no_spawn = Git { program: r.path.join("no-program"), ..r.git.clone() };
    assert_eq!(no_spawn.disk_usage_only::<&str>(&[]).unwrap(), 0);
    for name in ["main", "refs/a..b", "refs/a*", "refs/a@{1}"] {
        let error = no_spawn.disk_usage_only(&[name]).unwrap_err();
        assert_eq!(error.code, None);
        assert!(error.stderr.contains(name), "{error:?}");
        assert!(!error.stderr.contains("spawn"), "invalid name must refuse before launch: {error:?}");
    }
}

fn loose_size(r: &GitRepo, id: &Oid) -> u64 {
    let name = id.as_str();
    fs::metadata(r.path.join(".git/objects").join(&name[..2]).join(&name[2..])).unwrap().len()
}
fn run(git: &Git, args: &[&str]) -> Vec<u8> { git.run(args, &RunOpts::default()).unwrap().stdout }
fn observe(r: &GitRepo) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    (run(&r.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), fs::read(r.path.join(".git/index")).unwrap(),
     fs::read(r.path.join(".git/HEAD")).unwrap(), fs::read(r.path.join("base")).unwrap(), fs::read(r.path.join("untracked")).unwrap())
}
