//! Answers git gives that look odd but are what git prints: a remote with no url, and a URL ending in a CR byte.
use crate::fixture::{append_config, raw};
use crate::support::GitRepo;

#[test]
fn a_remote_with_no_url_answers_its_own_name() {
    let r = GitRepo::new();
    raw(&r, &["config", "remote.nourl.fetch", "+refs/heads/*:refs/remotes/nourl/*"]);
    assert_eq!(r.git.remote_push_urls("nourl"), Ok(vec!["nourl".to_string()]));
}

#[test]
fn a_cr_byte_inside_a_quoted_url_is_kept_untouched() {
    let r = GitRepo::new();
    append_config(&r, b"[remote \"crok\"]\n\turl = \"/tmp/cr\r\"\n");
    assert_eq!(r.git.remote_push_urls("crok"), Ok(vec!["/tmp/cr\r".to_string()]));
}
