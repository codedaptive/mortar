use crate::read_unmerged_paths::read_unmerged_paths;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

// Captured from git 2.56.0: `ls-files --unmerged -z --full-name -- :/` after a merge conflicting in three files, run
// at the worktree root and in its subfolder `sub` (both gave these same bytes).
const THREE_PATHS_THREE_STAGES: &[u8] = b"\
100644 df967b96a579e45a18b8251732d16804b2e56a55 1\tsp ace\0\
100644 ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2\tsp ace\0\
100644 e45c9c2666d44e0327c1f9c239a74c508336053e 3\tsp ace\0\
100644 df967b96a579e45a18b8251732d16804b2e56a55 1\tsub/inner.txt\0\
100644 ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2\tsub/inner.txt\0\
100644 e45c9c2666d44e0327c1f9c239a74c508336053e 3\tsub/inner.txt\0\
100644 df967b96a579e45a18b8251732d16804b2e56a55 1\ttop.txt\0\
100644 ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2\ttop.txt\0\
100644 e45c9c2666d44e0327c1f9c239a74c508336053e 3\ttop.txt\0";

// Captured the same way after an add/add conflict on `added` and on a name holding a newline (stages 2 and 3 only).
const ADD_ADD_WITH_A_NEWLINE: &[u8] = b"\
100644 28ce6a8b26aa170e1de65536fe8abe1832bd3242 2\tadded\0\
100644 13e7564ea0c889e81bcba6f8e496b2a74cdb32fa 3\tadded\0\
100644 28ce6a8b26aa170e1de65536fe8abe1832bd3242 2\tnl\nname\0\
100644 13e7564ea0c889e81bcba6f8e496b2a74cdb32fa 3\tnl\nname\0";

fn read(stdout: &[u8]) -> Vec<Vec<u8>> {
    read_unmerged_paths(stdout).unwrap().iter().map(|p| p.as_os_str().as_bytes().to_vec()).collect()
}

fn refusal(stdout: &[u8]) -> String {
    let err = read_unmerged_paths(stdout).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

#[test]
fn three_stages_of_one_path_give_one_path() {
    let stdout = b"\
100644 df967b96a579e45a18b8251732d16804b2e56a55 1\ttop.txt\0\
100644 ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2\ttop.txt\0\
100644 e45c9c2666d44e0327c1f9c239a74c508336053e 3\ttop.txt\0";
    assert_eq!(read(stdout), vec![b"top.txt".to_vec()]);
}

#[test]
fn captured_conflict_gives_each_path_once_in_gits_order() {
    assert_eq!(
        read(THREE_PATHS_THREE_STAGES),
        vec![b"sp ace".to_vec(), b"sub/inner.txt".to_vec(), b"top.txt".to_vec()]
    );
}

#[test]
fn a_path_with_stages_2_and_3_and_one_with_a_newline_are_each_named_once() {
    assert_eq!(read(ADD_ADD_WITH_A_NEWLINE), vec![b"added".to_vec(), b"nl\nname".to_vec()]);
}

#[test]
fn a_path_with_stages_1_to_3_then_one_with_stages_2_and_3() {
    let stdout = b"\
100644 df967b96a579e45a18b8251732d16804b2e56a55 1\tsp ace\0\
100644 ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2\tsp ace\0\
100644 e45c9c2666d44e0327c1f9c239a74c508336053e 3\tsp ace\0\
100644 28ce6a8b26aa170e1de65536fe8abe1832bd3242 2\ttop.txt\0\
100644 13e7564ea0c889e81bcba6f8e496b2a74cdb32fa 3\ttop.txt\0";
    assert_eq!(read_unmerged_paths(stdout).unwrap(), vec![PathBuf::from("sp ace"), PathBuf::from("top.txt")]);
}

#[test]
fn a_tab_after_the_first_belongs_to_the_path() {
    assert_eq!(read(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 1\ta\tb\0"), vec![b"a\tb".to_vec()]);
}

#[test]
fn non_utf8_path_is_kept() {
    assert_eq!(read(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 2\tcaf\xe9\0"), vec![b"caf\xe9".to_vec()]);
}

#[test]
fn a_sha256_object_id_is_read() {
    let stdout = b"100644 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef 3\tx\0";
    assert_eq!(read(stdout), vec![b"x".to_vec()]);
}

#[test]
fn empty_output_is_no_paths() {
    assert!(read_unmerged_paths(b"").unwrap().is_empty());
}

#[test]
fn a_last_record_without_its_nul_is_refused_naming_the_output() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 1\ttop.txt");
    assert!(stderr.contains("no final NUL"), "{stderr}");
    assert!(stderr.contains("\"100644 df967b96a579e45a18b8251732d16804b2e56a55 1\\ttop.txt\""), "{stderr}");
}

#[test]
fn a_record_without_a_tab_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 1 top.txt\0");
    assert!(stderr.contains("without a tab"), "{stderr}");
}

#[test]
fn a_header_of_two_fields_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55\ttop.txt\0");
    assert!(stderr.contains("three fields"), "{stderr}");
}

#[test]
fn a_header_of_four_fields_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 1 1\ttop.txt\0");
    assert!(stderr.contains("three fields"), "{stderr}");
}

#[test]
fn an_empty_mode_is_refused() {
    let stderr = refusal(b" df967b96a579e45a18b8251732d16804b2e56a55 1\ttop.txt\0");
    assert!(stderr.contains("empty mode"), "{stderr}");
}

#[test]
fn stage_0_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 0\ttop.txt\0");
    assert!(stderr.contains("stage"), "{stderr}");
}

#[test]
fn stage_4_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 4\ttop.txt\0");
    assert!(stderr.contains("stage"), "{stderr}");
}

#[test]
fn a_bad_object_id_is_refused() {
    let stderr = refusal(b"100644 DF967B96A579E45A18B8251732D16804B2E56A55 1\ttop.txt\0");
    assert!(stderr.contains("bad object id"), "{stderr}");
}

#[test]
fn a_short_object_id_is_refused() {
    let stderr = refusal(b"100644 df967b9 1\ttop.txt\0");
    assert!(stderr.contains("bad object id"), "{stderr}");
}

#[test]
fn an_empty_path_is_refused() {
    let stderr = refusal(b"100644 df967b96a579e45a18b8251732d16804b2e56a55 1\t\0");
    assert!(stderr.contains("empty path"), "{stderr}");
}

#[test]
fn a_lone_nul_is_refused() {
    assert!(read_unmerged_paths(b"\0").is_err());
}
