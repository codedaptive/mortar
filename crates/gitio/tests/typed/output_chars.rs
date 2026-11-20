use crate::fixture::{commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{BaselineQuery, RunOpts};
use std::fs;
use std::path::PathBuf;

fn history() -> (GitRepo, String, String) {
    let r = repo();
    let first_tree = tree(&r.git, &[("selected.txt", b"original\n"), ("other.txt", b"before\n")]);
    let first = commit(&r.git, &first_tree, &[], "first\n");
    let second_tree = tree(&r.git, &[
        ("selected.txt", "héllo wörld ✓ 日本語\n".as_bytes()),
        ("other.txt", b"a different second file with more changes\n"),
    ]);
    let second = commit(&r.git, &second_tree, std::slice::from_ref(&first), "second\n");
    set_ref(&r.git, "refs/heads/main", &second);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    (r, first.to_string(), second.to_string())
}

fn dirty(r: &GitRepo) {
    fs::write(r.path.join("selected.txt"), "dirty héllo ✓ 日本語\n").unwrap();
    fs::write(r.path.join("untracked.txt"), "untracked\n").unwrap();
}

fn plain(r: &GitRepo, args: &[&str]) -> String {
    let output = r.git.run(args, &RunOpts {
        cwd: Some(r.path.clone()), read_only: true, ..RunOpts::default()
    }).unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn log_patches_counts_characters_not_bytes() {
    let (r, first, second) = history();
    let range = format!("{first}..{second}");
    let expected = plain(&r, &["log", "-p", &range]);
    let count = r.git.output_chars(&r.path, &BaselineQuery::LogPatches {
        range, paths: vec![],
    }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
    assert!(count < u64::try_from(expected.len()).unwrap());
}

#[test]
fn diff_between_revisions_matches_plain_git() {
    let (r, first, second) = history();
    let expected = plain(&r, &["diff", &first, &second]);
    assert!(!expected.is_empty());
    let count = r.git.output_chars(&r.path, &BaselineQuery::Diff {
        a: first, b: second, paths: vec![],
    }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}

#[test]
fn diff_head_matches_plain_git_in_dirty_checkout() {
    let (r, _, _) = history();
    dirty(&r);
    let expected = plain(&r, &["diff", "HEAD"]);
    assert!(expected.contains("dirty héllo"));
    let count = r.git.output_chars(&r.path, &BaselineQuery::DiffHead { paths: vec![] }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}

#[test]
fn status_matches_plain_git_in_dirty_checkout() {
    let (r, _, _) = history();
    dirty(&r);
    let expected = plain(&r, &["status"]);
    assert!(expected.contains("selected.txt") && expected.contains("untracked.txt"));
    let count = r.git.output_chars(&r.path, &BaselineQuery::Status { paths: vec![] }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}

#[test]
fn paths_limit_the_diff() {
    let (r, first, second) = history();
    let expected = plain(&r, &["diff", &first, &second, "--", "selected.txt"]);
    let all = plain(&r, &["diff", &first, &second]);
    assert!(!expected.is_empty());
    assert!(expected.chars().count() < all.chars().count());
    let count = r.git.output_chars(&r.path, &BaselineQuery::Diff {
        a: first, b: second, paths: vec![PathBuf::from("selected.txt")],
    }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}

#[test]
fn large_log_patch_counts_across_stream_chunks() {
    let r = repo();
    let empty = tree(&r.git, &[]);
    let first = commit(&r.git, &empty, &[], "first\n");
    let content = "héllo wörld ✓ 日本語\n".repeat(12_000);
    let large = tree(&r.git, &[("large.txt", content.as_bytes())]);
    let second = commit(&r.git, &large, std::slice::from_ref(&first), "large\n");
    let range = format!("{first}..{second}");
    let expected = plain(&r, &["log", "-p", &range]);
    assert!(expected.len() > 64 * 1024);
    let count = r.git.output_chars(&r.path, &BaselineQuery::LogPatches {
        range, paths: vec![],
    }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}

#[test]
fn unknown_revision_returns_git_exit_and_stderr() {
    let (r, _, _) = history();
    let error = r.git.output_chars(&r.path, &BaselineQuery::LogPatches {
        range: "no-such-rev..HEAD".into(), paths: vec![],
    }).unwrap_err();
    assert!(matches!(error.code, Some(code) if code != 0));
    assert!(!error.stderr.is_empty());
}

#[test]
fn option_revision_is_refused_without_creating_output_file() {
    let (r, _, _) = history();
    let error = r.git.output_chars(&r.path, &BaselineQuery::LogPatches {
        range: "--output=x".into(), paths: vec![],
    }).unwrap_err();
    assert_eq!(error.code, None);
    assert!(!r.path.join("x").exists());
}

#[test]
fn malformed_revisions_and_empty_paths_are_refused_before_spawn() {
    let (r, _, _) = history();
    let no_spawn = gitio::Git { program: r.path.join("no-program"), ..r.git.clone() };
    let mut queries = Vec::new();
    for rev in ["", "-bad", "HEAD:file", "HEAD\0bad"] {
        queries.push(BaselineQuery::LogPatches { range: rev.into(), paths: vec![] });
        queries.push(BaselineQuery::Diff { a: rev.into(), b: "HEAD".into(), paths: vec![] });
        queries.push(BaselineQuery::Diff { a: "HEAD".into(), b: rev.into(), paths: vec![] });
    }
    queries.extend([
        BaselineQuery::LogPatches { range: "HEAD".into(), paths: vec![PathBuf::new()] },
        BaselineQuery::Diff { a: "HEAD".into(), b: "HEAD".into(), paths: vec![PathBuf::new()] },
        BaselineQuery::DiffHead { paths: vec![PathBuf::new()] },
        BaselineQuery::Status { paths: vec![PathBuf::new()] },
    ]);
    for query in queries {
        let error = no_spawn.output_chars(&r.path, &query).unwrap_err();
        assert_eq!(error.code, None, "{query:?}");
        assert!(!error.stderr.contains("no-program"), "{query:?}: {error:?}");
    }
}

#[test]
fn query_runs_in_the_requested_folder() {
    let (r, _, _) = history();
    let (other, _, _) = history();
    dirty(&other);
    let expected = plain(&other, &["diff", "HEAD"]);
    assert!(!expected.is_empty());
    assert!(plain(&r, &["diff", "HEAD"]).is_empty());
    let count = r.git.output_chars(&other.path, &BaselineQuery::DiffHead { paths: vec![] }).unwrap();
    assert_eq!(count, u64::try_from(expected.chars().count()).unwrap());
}
