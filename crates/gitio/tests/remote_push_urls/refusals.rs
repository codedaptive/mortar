//! Errors: refusals made before or after git runs (code None) and git's own exits (code Some).
use crate::fixture::append_config;
use crate::support::GitRepo;

#[test]
fn a_non_utf8_url_is_refused_not_returned_lossily() {
    let r = GitRepo::new();
    append_config(&r, b"[remote \"bad\"]\n\turl = /tmp/\xff\xfe.git\n");
    let Err(error) = r.git.remote_push_urls("bad") else {
        panic!("a non-UTF-8 URL must be an Err, never lossy text");
    };
    assert_eq!(error.code, None, "{error:?}");
}

#[test]
fn an_unknown_remote_is_gits_exit_2() {
    let r = GitRepo::new();
    let Err(error) = r.git.remote_push_urls("nosuch") else {
        panic!("an unknown remote must be an Err");
    };
    assert_eq!(error.code, Some(2), "{error:?}");
}

#[test]
fn a_remote_name_starting_with_a_dash_is_refused_before_git_runs() {
    let r = GitRepo::new();
    let Err(error) = r.git.remote_push_urls("-oops") else {
        panic!("a remote name starting with '-' must be refused");
    };
    assert_eq!(error.code, None, "{error:?}");
    assert!(error.args.is_empty(), "{error:?}");
}

#[test]
fn an_unreadable_config_is_gits_exit_128() {
    let r = GitRepo::new();
    append_config(&r, b"[remote \"cr\"]\n\turl = \"/tmp/cr\\r\"\n");
    let Err(error) = r.git.remote_push_urls("cr") else {
        panic!("a bad config line must be an Err");
    };
    assert_eq!(error.code, Some(128), "{error:?}");
}
