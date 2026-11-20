use crate::commit_tree_request::commit_tree_request;
use crate::{GitError, Oid, Person};
use std::ffi::OsString;

const TREE: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";
const P1: &str = "1111111111111111111111111111111111111111";
const P256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn person(name: &str, email: &str, date: Option<&str>) -> Person {
    Person {
        name: name.to_string(),
        email: email.to_string(),
        date: date.map(str::to_string),
    }
}

fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    items.iter().map(|(k, v)| (OsString::from(k), OsString::from(v))).collect()
}

fn refused(message: &str, author: Option<&Person>, committer: Option<&Person>) -> GitError {
    commit_tree_request(&oid(TREE), &[], message, author, committer).unwrap_err()
}

#[test]
fn no_parents_is_a_root_commit() {
    let (args, _) = commit_tree_request(&oid(TREE), &[], "m\n", None, None).unwrap();
    assert_eq!(args, vec![OsString::from("commit-tree"), OsString::from(TREE)]);
}

#[test]
fn each_parent_is_one_p_flag_in_order() {
    let (args, _) = commit_tree_request(&oid(TREE), &[oid(P256), oid(P1)], "m\n", None, None).unwrap();
    let expected: Vec<OsString> = [
        "commit-tree",
        "4c3115d7f09202012d6f64a6b6174c6774c3192e",
        "-p",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "-p",
        "1111111111111111111111111111111111111111",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn message_goes_to_stdin_as_given_and_the_run_writes() {
    let (_, opts) = commit_tree_request(&oid(TREE), &[], "subject\n\n  body\r\nno final newline", None, None).unwrap();
    assert_eq!(opts.stdin, Some(b"subject\n\n  body\r\nno final newline".to_vec()));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn no_people_set_no_env() {
    let (_, opts) = commit_tree_request(&oid(TREE), &[], "m\n", None, None).unwrap();
    assert!(opts.env.is_empty());
}

#[test]
fn author_and_committer_with_dates_set_all_six_keys() {
    let author = person("Ann", "ann@x.org", Some("1700000000 +0000"));
    let committer = person("Cy Doe", "cy@y.org", Some("2023-11-14T22:13:20Z"));
    let (_, opts) = commit_tree_request(&oid(TREE), &[], "m\n", Some(&author), Some(&committer)).unwrap();
    assert_eq!(
        opts.env,
        pairs(&[
            ("GIT_AUTHOR_NAME", "Ann"),
            ("GIT_AUTHOR_EMAIL", "ann@x.org"),
            ("GIT_AUTHOR_DATE", "1700000000 +0000"),
            ("GIT_COMMITTER_NAME", "Cy Doe"),
            ("GIT_COMMITTER_EMAIL", "cy@y.org"),
            ("GIT_COMMITTER_DATE", "2023-11-14T22:13:20Z"),
        ])
    );
}

#[test]
fn no_date_sets_no_date_key() {
    let author = person("Ann", "ann@x.org", None);
    let (_, opts) = commit_tree_request(&oid(TREE), &[], "m\n", Some(&author), None).unwrap();
    assert_eq!(opts.env, pairs(&[("GIT_AUTHOR_NAME", "Ann"), ("GIT_AUTHOR_EMAIL", "ann@x.org")]));
}

#[test]
fn committer_alone_sets_only_committer_keys() {
    let committer = person("Cy", "cy@y.org", Some("1700000000 +0100"));
    let (_, opts) = commit_tree_request(&oid(TREE), &[], "m\n", None, Some(&committer)).unwrap();
    assert_eq!(
        opts.env,
        pairs(&[
            ("GIT_COMMITTER_NAME", "Cy"),
            ("GIT_COMMITTER_EMAIL", "cy@y.org"),
            ("GIT_COMMITTER_DATE", "1700000000 +0100"),
        ])
    );
}

#[test]
fn nul_in_message_is_refused_naming_it() {
    let err = refused("sub\0ject", None, None);
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"sub\\0ject\""), "{}", err.stderr);
}

#[test]
fn nul_in_author_name_is_refused_naming_it() {
    let err = refused("m\n", Some(&person("A\0nn", "a@x", None)), None);
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"A\\0nn\""), "{}", err.stderr);
}

#[test]
fn nul_in_author_email_is_refused() {
    let err = refused("m\n", Some(&person("Ann", "a\0@x", None)), None);
    assert!(err.stderr.contains("\"a\\0@x\""), "{}", err.stderr);
}

#[test]
fn nul_in_author_date_is_refused() {
    let err = refused("m\n", Some(&person("Ann", "a@x", Some("1\0 +0000"))), None);
    assert!(err.stderr.contains("\"1\\0 +0000\""), "{}", err.stderr);
}

#[test]
fn nul_in_committer_field_is_refused() {
    let err = refused("m\n", None, Some(&person("Cy", "c@y", Some("now\0"))));
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"now\\0\""), "{}", err.stderr);
}
