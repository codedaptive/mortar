use crate::patch_id_diff_request::patch_id_diff_request;
use crate::Oid;
use std::ffi::OsString;

const SHA1: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn os(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    items.iter().map(|(k, v)| (OsString::from(k), OsString::from(v))).collect()
}

#[test]
fn argv_is_diff_tree_with_every_flag_pinned_and_the_commit_last() {
    let (args, _) = patch_id_diff_request(&Oid::parse(SHA1).unwrap());
    let expected = os(&[
        "diff-tree",
        "-p",
        "--root",
        "--diff-merges=first-parent",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--no-relative",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        "--diff-algorithm=myers",
        "--indent-heuristic",
        "--unified=3",
        "--ignore-submodules=none",
        "--submodule=short",
        "--full-index",
        "4c3115d7f09202012d6f64a6b6174c6774c3192e",
    ]);
    assert_eq!(args, expected);
}

#[test]
fn env_pins_config_replace_refs_grafts_and_attributes() {
    let (_, opts) = patch_id_diff_request(&Oid::parse(SHA1).unwrap());
    let expected = pairs(&[
        ("GIT_DIFF_OPTS", ""),
        ("GIT_CONFIG_COUNT", "0"),
        (
            "GIT_CONFIG_PARAMETERS",
            "'core.quotePath'='true' 'core.attributesFile'='/dev/null' 'diff.suppressBlankEmpty'='false' \
             'core.bigFileThreshold'='512m' 'advice.graftFileDeprecated'='false'",
        ),
        ("GIT_NO_REPLACE_OBJECTS", "1"),
        ("GIT_GRAFT_FILE", "/dev/null"),
        ("GIT_ATTR_NOSYSTEM", "1"),
        ("GIT_ATTR_SOURCE", "4c3115d7f09202012d6f64a6b6174c6774c3192e"),
    ]);
    assert_eq!(opts.env, expected);
}

#[test]
fn opts_are_read_only_with_no_stdin_cwd_or_answer_codes() {
    let (_, opts) = patch_id_diff_request(&Oid::parse(SHA1).unwrap());
    assert!(opts.read_only);
    assert_eq!(opts.stdin, None);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
}

#[test]
fn a_64_hex_commit_is_last_and_the_attribute_source() {
    let (args, opts) = patch_id_diff_request(&Oid::parse(SHA256).unwrap());
    assert_eq!(args[0], OsString::from("diff-tree"));
    assert_eq!(args.len(), 18);
    assert_eq!(args[17], OsString::from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
    assert_eq!(
        opts.env[6],
        (
            OsString::from("GIT_ATTR_SOURCE"),
            OsString::from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        )
    );
}
