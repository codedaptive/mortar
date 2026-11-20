use crate::patch_id_hash_request::patch_id_hash_request;
use std::ffi::OsString;

#[test]
fn argv_is_patch_id_stable() {
    let (args, _) = patch_id_hash_request(Vec::new());
    assert_eq!(args, vec![OsString::from("patch-id"), OsString::from("--stable")]);
}

#[test]
fn stdin_carries_the_diff_bytes_unchanged() {
    let (_, opts) = patch_id_hash_request(b"diff --git a/f b/f\n\xff\0-x\n+y\n".to_vec());
    assert_eq!(opts.stdin, Some(b"diff --git a/f b/f\n\xff\0-x\n+y\n".to_vec()));
}

#[test]
fn an_empty_diff_is_an_empty_stdin_not_none() {
    let (_, opts) = patch_id_hash_request(Vec::new());
    assert_eq!(opts.stdin, Some(Vec::new()));
}

#[test]
fn opts_are_read_only_with_no_env_cwd_or_answer_codes() {
    let (_, opts) = patch_id_hash_request(b"x".to_vec());
    assert!(opts.read_only);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
}
