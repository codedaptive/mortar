//! attempts_of: a request's try branches, sorted by k, each with the folder that has it checked out; other refs
//! and odd attempt numbers ignored.

use std::path::PathBuf;

use gitio::{Oid, Worktree};

use super::{attempts_of, Attempt};

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}

fn worktree(path: &str, branch: Option<&str>) -> Worktree {
    Worktree {
        path: PathBuf::from(path),
        head: Some(oid(A)),
        branch: branch.map(str::to_string),
        bare: false,
        detached: branch.is_none(),
        locked: false,
        prunable: false,
    }
}

#[test]
fn reads_attempts_sorted_by_k_with_their_folders() {
    let input = refs(&[
        (C, "refs/heads/try/z/r/10"),
        (A, "refs/heads/req/z/r"),
        (B, "refs/heads/try/z/r/2"),
        (A, "refs/heads/try/z/r/1"),
    ]);
    let trees = [
        worktree("/w/r", Some("refs/heads/req/z/r")),
        worktree("/w/tries/try-2", Some("refs/heads/try/z/r/2")),
        worktree("/w/tries/try-10", Some("refs/heads/try/z/r/10")),
        worktree("/w/detached", None),
    ];
    assert_eq!(
        attempts_of("z", "r", &input, &trees),
        vec![
            Attempt { k: 1, branch: "refs/heads/try/z/r/1".to_string(), tip: oid(A), folder: None },
            Attempt {
                k: 2,
                branch: "refs/heads/try/z/r/2".to_string(),
                tip: oid(B),
                folder: Some(PathBuf::from("/w/tries/try-2")),
            },
            Attempt {
                k: 10,
                branch: "refs/heads/try/z/r/10".to_string(),
                tip: oid(C),
                folder: Some(PathBuf::from("/w/tries/try-10")),
            },
        ]
    );
}

#[test]
fn ignores_other_requests_zones_and_shared_prefixes() {
    let input = refs(&[
        (A, "refs/heads/try/z/other/1"),
        (A, "refs/heads/try/y/r/1"),
        (A, "refs/heads/try/z/rx/1"),
        (A, "refs/heads/try/z/r"),
        (A, "refs/mortar/trysnap/z/r/1"),
        (A, "refs/heads/req/z/r/1"),
    ]);
    assert_eq!(attempts_of("z", "r", &input, &[]), Vec::<Attempt>::new());
}

#[test]
fn ignores_odd_attempt_numbers() {
    let input = refs(&[
        (A, "refs/heads/try/z/r/0"),
        (A, "refs/heads/try/z/r/01"),
        (A, "refs/heads/try/z/r/"),
        (A, "refs/heads/try/z/r/+1"),
        (A, "refs/heads/try/z/r/-1"),
        (A, "refs/heads/try/z/r/1x"),
        (A, "refs/heads/try/z/r/1/2"),
        (A, "refs/heads/try/z/r/4294967296"),
    ]);
    assert_eq!(attempts_of("z", "r", &input, &[]), Vec::<Attempt>::new());
}

#[test]
fn reads_the_largest_attempt_number() {
    let input = refs(&[(B, "refs/heads/try/z/r/4294967295")]);
    assert_eq!(
        attempts_of("z", "r", &input, &[]),
        vec![Attempt { k: 4294967295, branch: "refs/heads/try/z/r/4294967295".to_string(), tip: oid(B), folder: None }]
    );
}
