use crate::fixture::{blob, commit, repo, tree};
use gitio::{MergeTreeOutcome, Oid};
use std::path::PathBuf;

#[test]
fn merge_tree_combines_disjoint_changes_without_a_checkout() {
    // Joins Git::merge_tree -> real three-way merge -> independently constructed expected tree.
    let r = repo();
    let base_tree = tree(&r.git, &[("a", b"a base\n"), ("b", b"b base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let ours_tree = tree(&r.git, &[("a", b"a changed\n"), ("b", b"b base\n")]);
    let ours = commit(&r.git, &ours_tree, std::slice::from_ref(&base), "ours\n");
    let theirs_tree = tree(&r.git, &[("a", b"a base\n"), ("b", b"b changed\n")]);
    let theirs = commit(&r.git, &theirs_tree, std::slice::from_ref(&base), "theirs\n");
    let expected = tree(&r.git, &[("a", b"a changed\n"), ("b", b"b changed\n")]);
    assert_eq!(r.git.merge_tree(&base, None, &ours, &theirs).unwrap(), MergeTreeOutcome::Clean(expected));
    assert!(!r.path.join("a").exists());
    assert!(!r.path.join("b").exists());
}

#[test]
fn merge_tree_conflict_lists_preserve_spaces_newlines_and_conflicted_content() {
    // Joins Git::merge_tree -> real conflict exit 1 -> tree/file-list parsing.
    let r = repo();
    let paths = ["a conflict", "z\nconflict"];
    let base_tree = tree(&r.git, &[(paths[0], b"base\n"), (paths[1], b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let ours_tree = tree(&r.git, &[(paths[0], b"ours\n"), (paths[1], b"ours\n")]);
    let ours = commit(&r.git, &ours_tree, std::slice::from_ref(&base), "ours\n");
    let theirs_tree = tree(&r.git, &[(paths[0], b"theirs\n"), (paths[1], b"theirs\n")]);
    let theirs = commit(&r.git, &theirs_tree, std::slice::from_ref(&base), "theirs\n");
    match r.git.merge_tree(&base, Some(&base), &ours, &theirs).unwrap() {
        MergeTreeOutcome::Conflicted { tree, files } => {
            assert_eq!(files, paths.map(PathBuf::from).to_vec());
            for path in paths {
                let content = String::from_utf8(blob(&r.git, &tree, path)).unwrap();
                assert!(content.contains("<<<<<<<") && content.contains("ours\n") && content.contains("theirs\n"), "{content}");
            }
        }
        other => panic!("expected conflicts: {other:?}"),
    }
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    let error = r.git.merge_tree(&base, None, &ours, &missing).unwrap_err();
    // The contract requires a diagnostic error, not a particular non-zero exit code.
    assert!(error.code.is_some_and(|code| code != 0), "{error:?}");
    assert!(!error.stderr.is_empty());
}

#[test]
fn union_driver_is_honoured_only_when_the_attribute_source_contains_it() {
    // Joins Git::merge_tree's --attr-source choice with real .gitattributes merge=union.
    let r = repo();
    let base_tree = tree(&r.git, &[("log.txt", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let ours_tree = tree(&r.git, &[("log.txt", b"ours\n")]);
    let ours = commit(&r.git, &ours_tree, std::slice::from_ref(&base), "ours\n");
    let theirs_tree = tree(&r.git, &[("log.txt", b"theirs\n")]);
    let theirs = commit(&r.git, &theirs_tree, std::slice::from_ref(&base), "theirs\n");
    let attributes_tree = tree(&r.git, &[("log.txt", b"base\n"), (".gitattributes", b"log.txt merge=union\n")]);
    let attributes = commit(&r.git, &attributes_tree, std::slice::from_ref(&base), "attributes only\n");
    assert!(!r.path.join(".gitattributes").exists());
    assert!(matches!(r.git.merge_tree(&base, None, &ours, &theirs).unwrap(), MergeTreeOutcome::Conflicted { .. }));
    let merged = match r.git.merge_tree(&attributes, None, &ours, &theirs).unwrap() {
        MergeTreeOutcome::Clean(tree) => tree,
        other => panic!("attribute source did not enable union: {other:?}"),
    };
    assert_eq!(blob(&r.git, &merged, "log.txt"), b"ours\ntheirs\n");
}
