use crate::fixture::{commit, repo, set_ref, tree};
use gitio::{Person, RunOpts};
use std::path::PathBuf;

#[test]
fn commit_tree_preserves_parent_order_message_and_explicit_identities() {
    // Joins Git::commit_tree -> real commit object -> raw cat-file observation.
    let r = repo();
    let empty = tree(&r.git, &[]);
    let first = commit(&r.git, &empty, &[], "first parent\n");
    let second = commit(&r.git, &empty, &[], "second parent\n");
    let contents = tree(&r.git, &[("file", b"payload\n")]);
    let author = Person { name: "Ada Author".into(), email: "ada@example.invalid".into(), date: Some("1234567890 +0000".into()) };
    let committer = Person { name: "Cora Committer".into(), email: "cora@example.invalid".into(), date: Some("1234567999 +0000".into()) };
    let result = r.git.commit_tree(&contents, &[second.clone(), first.clone()], "subject\n\nbody\n", Some(&author), Some(&committer)).unwrap();
    let actual = r.git.run(&["cat-file", "commit", result.as_str()], &RunOpts::default()).unwrap();
    let expected = format!("tree {contents}\nparent {second}\nparent {first}\nauthor Ada Author <ada@example.invalid> 1234567890 +0000\ncommitter Cora Committer <cora@example.invalid> 1234567999 +0000\n\nsubject\n\nbody\n");
    assert_eq!(actual.stdout, expected.as_bytes());
    let default = r.git.commit_tree(&empty, &[], "default identities\n", None, None).unwrap();
    let raw = r.git.run(&["cat-file", "commit", default.as_str()], &RunOpts::default()).unwrap();
    let text = String::from_utf8(raw.stdout).unwrap();
    assert!(text.contains("author Fixture Author <author@example.invalid> 946684800 +0000\n"), "{text}");
    assert!(text.contains("committer Fixture Committer <committer@example.invalid> 946771200 +0000\n"), "{text}");
}

#[test]
fn rev_parse_peels_annotated_tag_and_list_refs_respects_prefix() {
    // Joins Git::rev_parse and Git::list_refs with independently created real refs/tag objects.
    let r = repo();
    let empty = tree(&r.git, &[]);
    let first = commit(&r.git, &empty, &[], "first\n");
    let second = commit(&r.git, &empty, &[first.clone()], "second\n");
    set_ref(&r.git, "refs/heads/main", &first);
    set_ref(&r.git, "refs/heads/topic", &second);
    r.git.run(&["tag", "-a", "v1", "-m", "annotated tag", first.as_str()], &RunOpts::default()).unwrap();
    assert_eq!(r.git.rev_parse("main").unwrap(), Some(first.clone()));
    assert_eq!(r.git.rev_parse("v1").unwrap(), Some(first.clone()));
    assert_eq!(r.git.rev_parse("refs/heads/missing").unwrap(), None);
    assert_eq!(r.git.rev_parse("--help").unwrap(), None);
    let heads = vec![(first, "refs/heads/main".into()), (second, "refs/heads/topic".into())];
    assert_eq!(r.git.list_refs("refs/heads/").unwrap(), heads);
    assert!(r.git.list_refs("refs/missing/").unwrap().is_empty());
    let all = r.git.list_refs("").unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(&all[..2], heads.as_slice());
    assert_eq!(all[2].1, "refs/tags/v1");
    let raw_tag = r.git.run(&["rev-parse", "refs/tags/v1"], &RunOpts::default()).unwrap();
    assert_eq!(all[2].0.as_str(), String::from_utf8(raw_tag.stdout).unwrap().trim_end());
    assert_ne!(all[2].0, heads[0].0, "list_refs must not peel an annotated tag");
}

#[test]
fn changed_files_keeps_both_rename_sides_and_literal_unusual_paths() {
    // Joins Git::changed_files -> raw tree differences -> NUL path parsing.
    let r = repo();
    let old_tree = tree(&r.git, &[("old name.txt", b"same\n"), ("z space.txt", b"before\n")]);
    let old = commit(&r.git, &old_tree, &[], "before\n");
    let new_tree = tree(&r.git, &[("new\nname.txt", b"same\n"), ("z space.txt", b"after\n"), ("-leading", b"new\n")]);
    let new = commit(&r.git, &new_tree, &[old.clone()], "after\n");
    assert_eq!(r.git.changed_files(&old, &new).unwrap(), vec![
        PathBuf::from("-leading"), PathBuf::from("new\nname.txt"),
        PathBuf::from("old name.txt"), PathBuf::from("z space.txt"),
    ]);
    assert!(r.git.changed_files(&new, &new).unwrap().is_empty());
}

#[test]
fn range_diff_returns_real_git_review_text_without_colour() {
    // Joins Git::range_diff -> real git's independent range-diff text oracle.
    let r = repo();
    let initial = tree(&r.git, &[("file", b"base\n")]);
    let base = commit(&r.git, &initial, &[], "base\n");
    let old_tree = tree(&r.git, &[("file", b"base\nchange\n")]);
    let old = commit(&r.git, &old_tree, &[base.clone()], "topic patch\n");
    let new_tree = tree(&r.git, &[("file", b"base\nchange\namended\n")]);
    let new = commit(&r.git, &new_tree, &[base.clone()], "topic patch\n");
    r.git.run(&["config", "color.ui", "always"], &RunOpts::default()).unwrap();
    let actual = r.git.range_diff(&base, &old, &new).unwrap();
    let oracle = r.git.run(&["range-diff", "--no-color", &format!("{base}..{old}"), &format!("{base}..{new}")], &RunOpts::default()).unwrap();
    assert_eq!(actual.as_bytes(), oracle.stdout);
    assert!(actual.contains("topic patch"), "{actual}");
    assert!(actual.contains(&old.as_str()[..7]) && actual.contains(&new.as_str()[..7]), "{actual}");
    assert!(!actual.contains('\u{1b}'), "{actual}");
}
