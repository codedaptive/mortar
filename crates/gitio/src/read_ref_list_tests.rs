use crate::read_ref_list::read_ref_list;

const A1: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";
const B256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn pair(oid: &str, refname: &str) -> (String, String) {
    (oid.to_string(), refname.to_string())
}

fn read(stdout: &[u8]) -> Vec<(String, String)> {
    read_ref_list(stdout)
        .unwrap()
        .into_iter()
        .map(|(oid, refname)| (oid.as_str().to_string(), refname))
        .collect()
}

#[test]
fn one_line_is_one_ref() {
    assert_eq!(
        read(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/base\n"),
        vec![pair(A1, "refs/heads/base")]
    );
}

#[test]
fn lines_keep_git_order() {
    assert_eq!(
        read(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/wt1\n0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef refs/heads/wt0\n"),
        vec![pair(A1, "refs/heads/wt1"), pair(B256, "refs/heads/wt0")]
    );
}

#[test]
fn refname_keeps_text_after_the_first_space() {
    assert_eq!(
        read(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/a b\n"),
        vec![pair(A1, "refs/heads/a b")]
    );
}

#[test]
fn empty_output_is_no_refs() {
    assert_eq!(read(b""), Vec::<(String, String)>::new());
}

#[test]
fn missing_final_newline_is_refused() {
    assert!(read_ref_list(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/base").is_err());
}

#[test]
fn line_without_a_space_is_refused_naming_it() {
    let err = read_ref_list(b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"4c3115d7f09202012d6f64a6b6174c6774c3192e\""), "{}", err.stderr);
}

#[test]
fn bad_oid_is_refused() {
    assert!(read_ref_list(b"4c3115d refs/heads/base\n").is_err());
}

#[test]
fn empty_refname_is_refused() {
    assert!(read_ref_list(b"4c3115d7f09202012d6f64a6b6174c6774c3192e \n").is_err());
}

#[test]
fn blank_line_is_refused() {
    assert!(read_ref_list(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/a\n\n").is_err());
}

#[test]
fn non_utf8_is_refused() {
    assert!(read_ref_list(b"4c3115d7f09202012d6f64a6b6174c6774c3192e refs/heads/caf\xe9\n").is_err());
}
