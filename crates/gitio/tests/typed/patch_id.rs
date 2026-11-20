use crate::fixture::{repo, tree};
use gitio::{Git, Oid, Person, RunOpts};

fn commit(git: &Git, files: &[(&str, &[u8])], parents: &[Oid], message: &str) -> Oid {
    let contents = tree(git, files);
    git.commit_tree(&contents, parents, message, None, None).unwrap()
}

#[test]
fn same_change_on_different_bases_ignores_commit_metadata() {
    let r = repo();
    let base_a = commit(&r.git, &[("f", b"1\n")], &[], "base a\n");
    let base_b = commit(&r.git, &[("f", b"1\n"), ("other", b"z\n")], &[], "base b\n");
    let author_a = Person { name: "First Author".into(), email: "first@example.invalid".into(), date: Some("1234567890 +0000".into()) };
    let author_b = Person { name: "Second Author".into(), email: "second@example.invalid".into(), date: Some("1334567890 +0000".into()) };
    let committer_a = Person { name: "First Committer".into(), email: "first-c@example.invalid".into(), date: Some("1234567990 +0000".into()) };
    let committer_b = Person { name: "Second Committer".into(), email: "second-c@example.invalid".into(), date: Some("1334567990 +0000".into()) };
    let tree_a = tree(&r.git, &[("f", b"2\n")]);
    let tree_b = tree(&r.git, &[("f", b"2\n"), ("other", b"z\n")]);
    let a = r.git.commit_tree(&tree_a, &[base_a], "first message\n", Some(&author_a), Some(&committer_a)).unwrap();
    let b = r.git.commit_tree(&tree_b, &[base_b], "second message\n", Some(&author_b), Some(&committer_b)).unwrap();
    assert_ne!(a, b);
    assert_eq!(r.git.patch_id(&a).unwrap().unwrap(), r.git.patch_id(&b).unwrap().unwrap());
}

#[test]
fn different_changes_have_different_patch_ids() {
    let r = repo();
    let base = commit(&r.git, &[("f", b"1\n")], &[], "base\n");
    let a = commit(&r.git, &[("f", b"2\n")], std::slice::from_ref(&base), "a\n");
    let b = commit(&r.git, &[("f", b"3\n")], &[base], "b\n");
    assert_ne!(r.git.patch_id(&a).unwrap().unwrap(), r.git.patch_id(&b).unwrap().unwrap());
}

#[test]
fn unchanged_tree_has_no_patch_id() {
    let r = repo();
    let base = commit(&r.git, &[("f", b"same\n")], &[], "base\n");
    let empty = commit(&r.git, &[("f", b"same\n")], &[base], "empty\n");
    assert_eq!(r.git.patch_id(&empty).unwrap(), None);
}

#[test]
fn root_is_compared_with_empty_tree() {
    let r = repo();
    let root = commit(&r.git, &[("f", b"content\n")], &[], "root\n");
    let empty_root = commit(&r.git, &[], &[], "empty root\n");
    let addition = commit(&r.git, &[("f", b"content\n")], std::slice::from_ref(&empty_root), "addition\n");
    assert_eq!(r.git.patch_id(&root).unwrap().unwrap(), r.git.patch_id(&addition).unwrap().unwrap());
    assert_eq!(r.git.patch_id(&empty_root).unwrap(), None);
}

#[test]
fn merge_is_compared_with_first_parent() {
    let r = repo();
    let base = commit(&r.git, &[("base", b"base\n")], &[], "base\n");
    let main = commit(&r.git, &[("base", b"base\n"), ("x", b"main\n")], std::slice::from_ref(&base), "main\n");
    let side = commit(&r.git, &[("base", b"base\n"), ("s", b"side\n")], &[base], "side\n");
    let merge = commit(&r.git, &[("base", b"base\n"), ("x", b"main\n"), ("s", b"side\n")], &[main.clone(), side.clone()], "merge\n");
    let empty_merge = commit(&r.git, &[("base", b"base\n"), ("x", b"main\n")], &[main, side.clone()], "empty merge\n");
    assert_eq!(r.git.patch_id(&merge).unwrap().unwrap(), r.git.patch_id(&side).unwrap().unwrap());
    assert_eq!(r.git.patch_id(&empty_merge).unwrap(), None);
}

#[test]
fn unknown_commit_is_an_error() {
    let r = repo();
    let missing = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    assert!(r.git.patch_id(&missing).is_err());
}

#[test]
fn hostile_diff_config_does_not_change_patch_id() {
    let r = repo();
    let before = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n\nold\n\nnine\nten\neleven\ntwelve\nthirteen\nfourteen\nfifteen\n";
    let after = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n\nnew\n\nnine\nten\neleven\ntwelve\nthirteen\nfourteen\nfifteen\n";
    let base = commit(&r.git, &[("é.txt", before)], &[], "base\n");
    let changed = commit(&r.git, &[("é.txt", after)], &[base], "change\n");
    let original = r.git.patch_id(&changed).unwrap().unwrap();
    for (key, value) in [
        ("diff.noprefix", "true"),
        ("diff.mnemonicPrefix", "true"),
        ("diff.algorithm", "histogram"),
        ("diff.renames", "copies"),
        ("color.ui", "always"),
        ("core.quotePath", "false"),
        ("diff.suppressBlankEmpty", "true"),
        ("diff.context", "10"),
        ("core.bigFileThreshold", "1"),
    ] {
        r.git.run(&["config", key, value], &RunOpts::default()).unwrap();
    }
    assert_eq!(original, r.git.patch_id(&changed).unwrap().unwrap());
}

#[test]
fn uncommitted_worktree_attributes_do_not_change_patch_id() {
    let r = repo();
    let base = commit(&r.git, &[("f.txt", b"before\n")], &[], "base\n");
    let changed = commit(&r.git, &[("f.txt", b"after\n")], &[base], "change\n");
    let original = r.git.patch_id(&changed).unwrap().unwrap();
    std::fs::write(r.git.repo.join(".gitattributes"), b"*.txt binary\n").unwrap();
    assert_eq!(original, r.git.patch_id(&changed).unwrap().unwrap());
}

#[test]
fn header_like_body_lines_contribute_to_patch_id() {
    let r = repo();
    let base = commit(&r.git, &[("f", b"-- x\n")], &[], "base\n");
    let header_like = commit(&r.git, &[("f", b"++ y\n")], std::slice::from_ref(&base), "header-like\n");
    let removal_only = commit(&r.git, &[("f", b"")], &[base], "removal only\n");
    let empty_base = commit(&r.git, &[("f", b"")], &[], "empty base\n");
    let addition_only = commit(&r.git, &[("f", b"++ y\n")], &[empty_base], "addition only\n");
    let header_like_id = r.git.patch_id(&header_like).unwrap().unwrap();
    assert_ne!(header_like_id, r.git.patch_id(&removal_only).unwrap().unwrap());
    assert_ne!(header_like_id, r.git.patch_id(&addition_only).unwrap().unwrap());
}

#[test]
fn hostile_environment_does_not_change_patch_id() {
    let r = repo();
    let before = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n\nold\n\ntwelve\nthirteen\nfourteen\nfifteen\nsixteen\nseventeen\neighteen\nnineteen\ntwenty\ntwenty-one\ntwenty-two\ntwenty-three\n";
    let after = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n\nnew\n\ntwelve\nthirteen\nfourteen\nfifteen\nsixteen\nseventeen\neighteen\nnineteen\ntwenty\ntwenty-one\ntwenty-two\ntwenty-three\n";
    let base = commit(&r.git, &[("é.txt", before)], &[], "base\n");
    let changed = commit(&r.git, &[("é.txt", after)], &[base], "change\n");
    let original = r.git.patch_id(&changed).unwrap().unwrap();

    let mut hostile = r.git.clone();
    for (key, value) in [
        ("GIT_DIFF_OPTS", "-u9"),
        ("GIT_CONFIG_PARAMETERS", "'core.quotePath'='false' 'diff.suppressBlankEmpty'='true'"),
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "core.bigFileThreshold"),
        ("GIT_CONFIG_VALUE_0", "1"),
    ] {
        hostile.env.push((key.into(), value.into()));
    }
    assert_eq!(original, hostile.patch_id(&changed).unwrap().unwrap());
}
