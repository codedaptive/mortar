//! Zone integration tests for land::rereview on throwaway repositories: an edit after review shows
//! only the edit, a remake on a moved origin shows no change of its own, identical squashes say nothing changed,
//! and two different changes are refused. Setup uses plain git with a fixed author, committer and date, so every
//! run builds the same commits; rereview's own git runs ignore the machine's global and system git config.

use std::path::Path;
use std::process::Command;

use gitio::{Git, Oid};
use land::{rereview, RereviewError};
use testkit::TestRepo;

const CHANGE_X: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";
const CHANGE_Y: &str = "zkzkzkzkzkzkzkzkzkzkzkzkzkzkzkzk";
// Eight lines, so a one-line edit stays within range-diff's default creation factor and the two versions pair;
// with three lines the edit outweighs the patch and git shows the old and new versions as unpaired `<` and `>`.
const REVIEWED_F: &str = "a\nb\nc\nd\ne\nf\ng\nh\n";
const EDITED_F: &str = "a\nB2\nc\nd\ne\nf\ng\nh\n";

/// Runs git in `repo` with the fixture's fixed identity and no user or system config; its trimmed stdout.
fn git(repo: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo).args(args);
    // The fixture must target this repository even when the test runs inside a git hook.
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX", "GIT_OBJECT_DIRECTORY"] {
        command.env_remove(name);
    }
    let output = command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// A commit on `parent` writing each (path, content) and carrying `subject` and the Mortar-Change-Id `change`.
fn commit_on(repo: &Path, parent: &str, files: &[(&str, &str)], subject: &str, change: &str) -> String {
    git(repo, &["checkout", "-q", "--detach", parent]);
    for (path, content) in files {
        std::fs::write(repo.join(path), content).unwrap();
        git(repo, &["add", path]);
    }
    let trailer = format!("Mortar-Change-Id: {change}");
    git(repo, &["commit", "-q", "-m", subject, "-m", &trailer]);
    git(repo, &["rev-parse", "HEAD"])
}

/// A repository whose HEAD holds base B (one file `base`); B's id.
fn base(repo: &TestRepo) -> String {
    std::fs::write(repo.path.join("base"), "base\n").unwrap();
    git(&repo.path, &["add", "base"]);
    git(&repo.path, &["commit", "-q", "-m", "base"]);
    git(&repo.path, &["rev-parse", "HEAD"])
}

/// The gitio value rereview runs with. Its runs ignore the machine's global and system git config, so settings
/// such as core.hooksPath or diff options cannot reach the throwaway repository or change range-diff's output.
fn git_of(path: &Path) -> Git {
    let env = vec![("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into())];
    Git { program: "git".into(), repo: path.to_path_buf(), env }
}

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

/// The range-diff's pair lines: each starts with `1:` or `-:`, then the pair mark.
fn pair_lines(text: &str) -> Vec<&str> {
    text.lines().filter(|line| line.starts_with("1:") || line.starts_with("-:")).collect()
}

#[test]
fn an_edit_after_review_shows_only_the_edit() {
    let repo = TestRepo::new().unwrap();
    let b = base(&repo);
    let reviewed = commit_on(&repo.path, &b, &[("f", REVIEWED_F)], "add f", CHANGE_X);
    let new = commit_on(&repo.path, &b, &[("f", EDITED_F)], "add f", CHANGE_X);

    let text = rereview(&git_of(&repo.path), &oid(&reviewed), &oid(&new)).unwrap();

    let pairs = pair_lines(&text);
    assert_eq!(pairs.len(), 1, "{text}");
    assert!(pairs[0].starts_with("1:") && pairs[0].contains(" ! "), "{text}");
    let nested: Vec<&str> = text.lines().map(str::trim).collect();
    assert!(nested.contains(&"-+b"), "the removed line b is not shown: {text}");
    assert!(nested.contains(&"++B2"), "the added line B2 is not shown: {text}");
    for unchanged in ["-+a", "++a", "-+c", "++c"] {
        assert!(!nested.contains(&unchanged), "{unchanged} shown as changed: {text}");
    }
}

#[test]
fn a_remake_on_a_moved_origin_shows_no_change_of_its_own() {
    let repo = TestRepo::new().unwrap();
    let b = base(&repo);
    let reviewed = commit_on(&repo.path, &b, &[("f", REVIEWED_F)], "add f", CHANGE_X);
    let moved = commit_on(&repo.path, &b, &[("unrelated", "other\n")], "origin moved", CHANGE_Y);
    let new = commit_on(&repo.path, &moved, &[("f", REVIEWED_F)], "add f", CHANGE_X);

    let text = rereview(&git_of(&repo.path), &oid(&reviewed), &oid(&new)).unwrap();

    let pairs = pair_lines(&text);
    assert_eq!(pairs.len(), 1, "{text}");
    assert!(pairs[0].starts_with("1:") && pairs[0].contains(" = "), "{text}");
    for mark in [" ! ", " < ", " > "] {
        assert!(!pairs[0].contains(mark), "pair marked {mark:?}: {text}");
    }
}

#[test]
fn identical_squashes_say_nothing_changed() {
    let repo = TestRepo::new().unwrap();
    let b = base(&repo);
    let reviewed = commit_on(&repo.path, &b, &[("f", REVIEWED_F)], "add f", CHANGE_X);
    // The same tree and parent and one change ID under a different message: a second commit, not a second name.
    let tree = git(&repo.path, &["rev-parse", &format!("{reviewed}^{{tree}}")]);
    let trailer = format!("Mortar-Change-Id: {CHANGE_X}");
    let reworded = git(&repo.path, &["commit-tree", &tree, "-p", &b, "-m", "add f, reworded", "-m", &trailer]);
    assert_ne!(reworded, reviewed);

    let same = rereview(&git_of(&repo.path), &oid(&reviewed), &oid(&reviewed)).unwrap();
    let recommitted = rereview(&git_of(&repo.path), &oid(&reviewed), &oid(&reworded)).unwrap();

    assert_eq!(same, format!("nothing changed: {reviewed} has the same tree and parents as {reviewed}\n"));
    assert_eq!(recommitted, format!("nothing changed: {reworded} has the same tree and parents as {reviewed}\n"));
}

#[test]
fn two_different_changes_are_refused() {
    let repo = TestRepo::new().unwrap();
    let b = base(&repo);
    let reviewed = commit_on(&repo.path, &b, &[("f", REVIEWED_F)], "add f", CHANGE_X);
    let new = commit_on(&repo.path, &b, &[("f", EDITED_F)], "add f", CHANGE_Y);

    let refused = rereview(&git_of(&repo.path), &oid(&reviewed), &oid(&new));

    assert!(matches!(refused, Err(RereviewError::NotSameChange { .. })), "{refused:?}");
}
