use crate::read_merge_tree::read_merge_tree;
use crate::{GitError, MergeTreeOutcome, Oid, Output};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

const T1: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";
const T256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn exit_0_is_clean_with_the_tree() {
    let out = output(0, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0", b"");
    assert_eq!(read_merge_tree(&out), Ok(MergeTreeOutcome::Clean(oid(T1))));
}

#[test]
fn exit_1_with_one_file_is_conflicted_and_messages_are_not_read() {
    let stdout = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\0a/f\0\x001\0a/f\0Auto-merging\0Auto-merging a/f\n\0";
    assert_eq!(
        read_merge_tree(&output(1, stdout, b"")),
        Ok(MergeTreeOutcome::Conflicted {
            tree: oid(T256),
            files: vec![PathBuf::from("a/f")],
        })
    );
}

#[test]
fn exit_1_with_two_files_lists_both_in_order() {
    let stdout = b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0b/g\0top\0\x001\0b/g\0CONFLICT (contents)\0x\n\0";
    assert_eq!(
        read_merge_tree(&output(1, stdout, b"")),
        Ok(MergeTreeOutcome::Conflicted {
            tree: oid(T1),
            files: vec![PathBuf::from("b/g"), PathBuf::from("top")],
        })
    );
}

#[test]
fn conflicted_names_keep_spaces_newlines_and_non_utf8() {
    let stdout = b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0sp ace\0nl\nname\0caf\xe9\0\0";
    let Ok(MergeTreeOutcome::Conflicted { files, .. }) = read_merge_tree(&output(1, stdout, b"")) else {
        panic!("not Conflicted");
    };
    let bytes: Vec<&[u8]> = files.iter().map(|f| f.as_os_str().as_bytes()).collect();
    assert_eq!(bytes, vec![&b"sp ace"[..], &b"nl\nname"[..], &b"caf\xe9"[..]]);
}

#[test]
fn exit_1_without_a_tree_is_gits_error_with_its_stderr() {
    let out = output(1, b"", b"merge-tree: nosuch - not something we can merge\n");
    assert_eq!(
        read_merge_tree(&out),
        Err(GitError {
            args: Vec::new(),
            code: Some(1),
            stderr: "merge-tree: nosuch - not something we can merge\n".to_string(),
        })
    );
}

#[test]
fn exit_1_with_a_bad_first_field_is_gits_error_not_a_conflict() {
    let err = read_merge_tree(&output(1, b"4c3115d\0a/f\0\0", b"fatal: x\n")).unwrap_err();
    assert_eq!(err.code, Some(1));
    assert_eq!(err.stderr, "fatal: x\n");
}

#[test]
fn exit_1_with_no_file_names_is_refused() {
    let err = read_merge_tree(&output(1, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0\0", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
}

#[test]
fn exit_1_with_an_unended_name_list_is_refused_naming_it() {
    let err = read_merge_tree(&output(1, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0a/f\0", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("merge-tree conflict"), "{}", err.stderr);
}

#[test]
fn exit_1_with_a_name_lacking_its_nul_is_refused() {
    let err = read_merge_tree(&output(1, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0a/f", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"a/f\""), "{}", err.stderr);
}

#[test]
fn exit_0_without_a_tree_is_refused_naming_stdout() {
    let err = read_merge_tree(&output(0, b"garbage", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 0"), "{}", err.stderr);
    assert!(err.stderr.contains("\"garbage\""), "{}", err.stderr);
}

#[test]
fn exit_0_with_a_tree_lacking_its_nul_is_refused() {
    assert!(read_merge_tree(&output(0, b"4c3115d7f09202012d6f64a6b6174c6774c3192e", b"")).is_err());
}

#[test]
fn other_exit_is_refused_naming_it() {
    let err = read_merge_tree(&output(2, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\0", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 2"), "{}", err.stderr);
}
