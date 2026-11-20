//! Joins Git::range_diff_ranges -> range_diff_ranges_request -> Git::run -> real git range-diff over two ranges.
//! Setup uses plain git commands.

#[path = "support/mod.rs"]
mod support;
// The shared fixture also carries blob and set_ref, which this target does not need.
#[allow(unused)]
#[path = "typed/fixture/mod.rs"]
mod fixture;

use fixture::{commit, repo, tree};
use gitio::{Oid, RunOpts};
use support::GitRepo;

/// origin0 -> R (the reviewed patch) and origin0 -> origin1 (origin moved) -> N (the patch remade on it).
struct Series {
    repo: GitRepo,
    origin0: Oid,
    reviewed: Oid,
    origin1: Oid,
    new: Oid,
}

fn series(new_a: &[u8]) -> Series {
    let repo = repo();
    let git = &repo.git;
    let origin0 = commit(git, &tree(git, &[("a", b"base\n")]), &[], "base\n");
    let reviewed = commit(git, &tree(git, &[("a", b"base\nchange\n")]), std::slice::from_ref(&origin0), "topic patch\n");
    let origin1_tree = tree(git, &[("a", b"base\n"), ("b", b"other\n")]);
    let origin1 = commit(git, &origin1_tree, std::slice::from_ref(&origin0), "origin moved\n");
    let new_tree = tree(git, &[("a", new_a), ("b", b"other\n")]);
    let new = commit(git, &new_tree, std::slice::from_ref(&origin1), "topic patch\n");
    git.run(&["config", "color.ui", "always"], &RunOpts::default()).unwrap();
    Series { repo, origin0, reviewed, origin1, new }
}

#[test]
fn moved_base_amended_squash_pairs_with_the_reviewed_one_as_changed() {
    let s = series(b"base\nchange\namended\n");
    let actual = s.repo.git.range_diff_ranges(&s.origin0, &s.reviewed, &s.origin1, &s.new).unwrap();

    assert_eq!(
        actual.lines().next(),
        Some(format!("1:  {} ! 1:  {} topic patch", &s.reviewed.as_str()[..7], &s.new.as_str()[..7]).as_str()),
        "{actual}"
    );
    assert!(!actual.lines().any(|line| line.contains("-:  -------")), "{actual}");
    assert!(!actual.contains('\u{1b}'), "{actual}");
    let oracle = s.repo.git.run(
        &[
            "range-diff",
            "--no-color",
            &format!("{}..{}", s.origin0, s.reviewed),
            &format!("{}..{}", s.origin1, s.new),
        ],
        &RunOpts::default(),
    ).unwrap();
    assert_eq!(actual.as_bytes(), oracle.stdout);
}

#[test]
fn moved_base_same_patch_pairs_as_equal_and_nothing_else() {
    let s = series(b"base\nchange\n");
    let actual = s.repo.git.range_diff_ranges(&s.origin0, &s.reviewed, &s.origin1, &s.new).unwrap();

    assert_eq!(
        actual,
        format!("1:  {} = 1:  {} topic patch\n", &s.reviewed.as_str()[..7], &s.new.as_str()[..7])
    );
}
