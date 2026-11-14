use crate::disk_usage_only_request::disk_usage_only_request;
use std::ffi::OsString;

fn os(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

/// The refusal for `refs`: no exit code, and its message holds `named`'s quoted ({:?}) form.
fn assert_refused_naming(refs: &[&str], named: &str) {
    let err = disk_usage_only_request(refs).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains(&format!("{named:?}")), "{}", err.stderr);
}

#[test]
fn argv_for_one_ref() {
    let (args, _) = disk_usage_only_request(&["refs/heads/side"]).unwrap();
    let want = os(&[
        "rev-list",
        "--objects",
        "--disk-usage",
        "--not",
        "--exclude=refs/heads/side",
        "--all",
        "--not",
        "--end-of-options",
        "refs/heads/side",
        "--",
    ]);
    assert_eq!(args, want);
}

#[test]
fn argv_for_two_refs_keeps_their_order() {
    let (args, _) = disk_usage_only_request(&["refs/mortar/snap/z/r", "refs/heads/a"]).unwrap();
    let want = os(&[
        "rev-list",
        "--objects",
        "--disk-usage",
        "--not",
        "--exclude=refs/mortar/snap/z/r",
        "--exclude=refs/heads/a",
        "--all",
        "--not",
        "--end-of-options",
        "refs/mortar/snap/z/r",
        "refs/heads/a",
        "--",
    ]);
    assert_eq!(args, want);
}

#[test]
fn string_refs_are_accepted_too() {
    let refs = vec![String::from("refs/heads/side")];
    let (args, _) = disk_usage_only_request(&refs).unwrap();
    assert_eq!(args[4], OsString::from("--exclude=refs/heads/side"));
    assert_eq!(args[8], OsString::from("refs/heads/side"));
}

#[test]
fn opts_are_read_only_with_128_as_an_answer() {
    let (_, opts) = disk_usage_only_request(&["refs/heads/side"]).unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![128]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn empty_name_is_refused() {
    assert_refused_naming(&[""], "");
}

#[test]
fn name_with_a_nul_is_refused() {
    assert_refused_naming(&["refs/heads/a\0b"], "refs/heads/a\0b");
}

#[test]
fn short_name_is_refused() {
    assert_refused_naming(&["main"], "main");
}

#[test]
fn range_is_refused() {
    assert_refused_naming(&["refs/heads/a..refs/heads/b"], "refs/heads/a..refs/heads/b");
}

#[test]
fn reflog_syntax_is_refused() {
    assert_refused_naming(&["refs/heads/x@{1}"], "refs/heads/x@{1}");
}

#[test]
fn each_revision_or_glob_character_is_refused() {
    for name in [
        "refs/heads/a^b",
        "refs/heads/a~b",
        "refs/heads/a:b",
        "refs/heads/a?b",
        "refs/heads/a*b",
        "refs/heads/a[b",
        "refs/heads/a\\b",
    ] {
        assert_refused_naming(&[name], name);
    }
}

#[test]
fn a_bad_ref_after_a_good_one_is_refused() {
    assert_refused_naming(&["refs/heads/side", "main"], "main");
}

#[test]
fn the_first_bad_ref_is_the_one_named() {
    let err = disk_usage_only_request(&["refs/heads/side", "main", "refs/heads/s*"]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"main\""), "{}", err.stderr);
    assert!(!err.stderr.contains("s*"), "{}", err.stderr);
}

#[test]
fn quote_and_dash_inside_a_full_name_are_accepted() {
    let (args, _) = disk_usage_only_request(&["refs/heads/q'x", "refs/heads/-x"]).unwrap();
    assert_eq!(args[4], OsString::from("--exclude=refs/heads/q'x"));
    assert_eq!(args[5], OsString::from("--exclude=refs/heads/-x"));
    assert_eq!(args[9], OsString::from("refs/heads/q'x"));
    assert_eq!(args[10], OsString::from("refs/heads/-x"));
}
