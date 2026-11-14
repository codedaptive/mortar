//! Real-git joins for fetch_ref and ls_remote against a local bare remote made inside the throwaway repo: request,
//! Git::run and reader together. Setup and every local observation use plain git, never the method under test.

use crate::fixture::{commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{FetchRefOutcome, Git, Oid, RunOpts};

/// A throwaway repo and its bare remote `remote.git`, which holds main (M), side (S, unrelated to M), an annotated
/// tag v1 on M, and HEAD pointing at main. Answers (repo, remote path, M, S, the tag object's id).
fn with_remote() -> (GitRepo, String, Oid, Oid, Oid) {
    let r = repo();
    let main = commit(&r.git, &tree(&r.git, &[("a", b"main\n")]), &[], "main\n");
    let side = commit(&r.git, &tree(&r.git, &[("a", b"side\n")]), &[], "side\n");
    let path = r.path.join("remote.git").to_str().unwrap().to_string();
    r.git.run(&["init", "-q", "--bare", &path], &RunOpts::default()).unwrap();
    let pushed = [format!("{main}:refs/heads/main"), format!("{side}:refs/heads/side")];
    r.git.run(&["push", "-q", &path, &pushed[0], &pushed[1]], &RunOpts::default()).unwrap();
    let remote = Git { repo: path.clone().into(), ..r.git.clone() };
    remote.run(&["symbolic-ref", "HEAD", "refs/heads/main"], &RunOpts::default()).unwrap();
    remote.run(&["tag", "-a", "-m", "v1", "v1", main.as_str()], &RunOpts::default()).unwrap();
    let tag = rev_parse(&remote, "refs/tags/v1");
    (r, path, main, side, tag)
}

/// The id a ref holds, read with plain `rev-parse --verify`.
fn rev_parse(git: &Git, name: &str) -> Oid {
    let out = git.run(&["rev-parse", "--verify", name], &RunOpts::default()).unwrap();
    Oid::parse(String::from_utf8(out.stdout).unwrap().trim_end()).unwrap()
}

fn listed(pairs: &[(&Oid, &str)]) -> Vec<(Oid, String)> {
    pairs.iter().map(|(oid, name)| ((*oid).clone(), name.to_string())).collect()
}

#[test]
fn fetch_ref_forces_dst_onto_the_remote_src_over_an_unrelated_commit() {
    let (r, remote, main, side, _) = with_remote();
    set_ref(&r.git, "refs/x/main", &side);
    assert_eq!(r.git.fetch_ref(&remote, "refs/heads/main", "refs/x/main"), Ok(FetchRefOutcome::Fetched));
    assert_eq!(rev_parse(&r.git, "refs/x/main"), main);
}

#[test]
fn fetch_ref_answers_missing_source_and_leaves_dst_untouched() {
    let (r, remote, _, side, _) = with_remote();
    set_ref(&r.git, "refs/x/main", &side);
    assert_eq!(r.git.fetch_ref(&remote, "refs/heads/nosuch", "refs/x/main"), Ok(FetchRefOutcome::MissingSource));
    assert_eq!(rev_parse(&r.git, "refs/x/main"), side);
}

#[test]
fn fetch_ref_fetches_no_tags() {
    let (r, remote, main, _, _) = with_remote();
    assert_eq!(r.git.fetch_ref(&remote, "refs/heads/main", "refs/x/main"), Ok(FetchRefOutcome::Fetched));
    assert_eq!(rev_parse(&r.git, "refs/x/main"), main);
    // show-ref --verify exits 1 for a ref that does not exist; the annotated tag on main must not have followed it.
    let probe = RunOpts { answer_codes: vec![1], read_only: true, ..RunOpts::default() };
    let tag = r.git.run(&["show-ref", "--verify", "--quiet", "refs/tags/v1"], &probe).unwrap();
    assert_eq!(tag.status, 1);
}

#[test]
fn fetch_ref_from_a_remote_that_does_not_exist_is_git_error_not_missing_source() {
    let r = repo();
    let missing = r.path.join("nosuch.git").to_str().unwrap().to_string();
    let err = r.git.fetch_ref(&missing, "refs/heads/main", "refs/x/main").unwrap_err();
    assert_eq!(err.code, Some(128));
    assert!(err.stderr.contains("does not appear to be a git repository"), "{}", err.stderr);
}

#[test]
fn ls_remote_lists_one_full_ref() {
    let (r, remote, main, _, _) = with_remote();
    let got = r.git.ls_remote(&remote, &["refs/heads/main".to_string()]).unwrap();
    assert_eq!(got, listed(&[(&main, "refs/heads/main")]));
}

#[test]
fn ls_remote_matches_short_patterns_against_the_refname_tail() {
    let (r, remote, main, _, _) = with_remote();
    for pattern in ["main", "heads/main"] {
        let got = r.git.ls_remote(&remote, &[pattern.to_string()]).unwrap();
        assert_eq!(got, listed(&[(&main, "refs/heads/main")]), "pattern {pattern}");
    }
}

#[test]
fn ls_remote_lists_several_patterns_in_refname_order() {
    let (r, remote, main, side, _) = with_remote();
    let got = r.git.ls_remote(&remote, &["side".to_string(), "main".to_string()]).unwrap();
    assert_eq!(got, listed(&[(&main, "refs/heads/main"), (&side, "refs/heads/side")]));
}

#[test]
fn ls_remote_lists_an_annotated_tag_and_its_peeled_line_for_a_glob() {
    let (r, remote, main, _, tag) = with_remote();
    let got = r.git.ls_remote(&remote, &["tags/*".to_string()]).unwrap();
    assert_eq!(got, listed(&[(&tag, "refs/tags/v1"), (&main, "refs/tags/v1^{}")]));
}

#[test]
fn ls_remote_lists_an_annotated_tag_alone_for_its_exact_name() {
    // The peeled line's name `refs/tags/v1^{}` does not end in `v1`, so an exact pattern does not match it.
    let (r, remote, _, _, tag) = with_remote();
    let got = r.git.ls_remote(&remote, &["v1".to_string()]).unwrap();
    assert_eq!(got, listed(&[(&tag, "refs/tags/v1")]));
}

#[test]
fn ls_remote_answers_empty_for_a_pattern_matching_nothing() {
    let (r, remote, _, _, _) = with_remote();
    assert_eq!(r.git.ls_remote(&remote, &["nosuch".to_string()]).unwrap(), Vec::new());
}

#[test]
fn ls_remote_without_patterns_lists_every_ref_and_head() {
    let (r, remote, main, side, tag) = with_remote();
    let got = r.git.ls_remote(&remote, &[]).unwrap();
    let want = [
        (&main, "HEAD"),
        (&main, "refs/heads/main"),
        (&side, "refs/heads/side"),
        (&tag, "refs/tags/v1"),
        (&main, "refs/tags/v1^{}"),
    ];
    assert_eq!(got, listed(&want));
}
