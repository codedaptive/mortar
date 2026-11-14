//! Remotes with ordinary URLs: pushurl over url, config order, and both URL rewrites applied as git applies them.
use crate::fixture::raw;
use crate::support::GitRepo;

#[test]
fn several_pushurls_come_in_config_order_without_the_url() {
    let r = GitRepo::new();
    raw(&r, &["remote", "add", "origin", "https://fetch.example/o.git"]);
    raw(&r, &["config", "--add", "remote.origin.pushurl", "https://push1.example/o.git"]);
    raw(&r, &["config", "--add", "remote.origin.pushurl", "ssh://push2.example/o.git"]);
    assert_eq!(
        r.git.remote_push_urls("origin"),
        Ok(vec!["https://push1.example/o.git".to_string(), "ssh://push2.example/o.git".to_string()])
    );
}

#[test]
fn a_remote_with_only_a_url_pushes_to_that_url() {
    let r = GitRepo::new();
    raw(&r, &["remote", "add", "origin", "https://only.example/u.git"]);
    assert_eq!(r.git.remote_push_urls("origin"), Ok(vec!["https://only.example/u.git".to_string()]));
}

#[test]
fn insteadof_rewrites_the_url() {
    let r = GitRepo::new();
    raw(&r, &["remote", "add", "rw", "short:z.git"]);
    raw(&r, &["config", "url.https://rewritten.example/.insteadOf", "short:"]);
    assert_eq!(r.git.remote_push_urls("rw"), Ok(vec!["https://rewritten.example/z.git".to_string()]));
}

#[test]
fn pushinsteadof_rewrites_the_url_for_push() {
    let r = GitRepo::new();
    raw(&r, &["remote", "add", "pi", "https://pi.example/w.git"]);
    raw(&r, &["config", "url.ssh://pushrw.example/.pushInsteadOf", "https://pi.example/"]);
    assert_eq!(r.git.remote_push_urls("pi"), Ok(vec!["ssh://pushrw.example/w.git".to_string()]));
}

#[test]
fn a_url_holding_spaces_is_kept_exactly() {
    let r = GitRepo::new();
    raw(&r, &["remote", "add", "sp", "/tmp/has space/repo.git"]);
    assert_eq!(r.git.remote_push_urls("sp"), Ok(vec!["/tmp/has space/repo.git".to_string()]));
}
