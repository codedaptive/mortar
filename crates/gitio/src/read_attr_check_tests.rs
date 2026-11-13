use crate::read_attr_check::read_attr_check;
use std::path::PathBuf;

/// git 2.56's answer for seven paths, as captured.
const SEVEN: &[u8] = b"a.txt\x00merge\x00union\x00b.dat\x00merge\x00unset\x00c.dat\x00merge\x00set\x00none\x00merge\x00unspecified\x00n\nl.txt\x00merge\x00union\x00-dash.txt\x00merge\x00union\x00sp ace\x00merge\x00unspecified\x00";

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn seven() -> Vec<PathBuf> {
    paths(&["a.txt", "b.dat", "c.dat", "none", "n\nl.txt", "-dash.txt", "sp ace"])
}

fn pair(path: &str, value: &str) -> (PathBuf, String) {
    (PathBuf::from(path), value.to_string())
}

#[test]
fn seven_captured_triples_pair_each_path_with_its_value() {
    assert_eq!(
        read_attr_check(SEVEN, &seven()).unwrap(),
        vec![
            pair("a.txt", "union"),
            pair("b.dat", "unset"),
            pair("c.dat", "set"),
            pair("none", "unspecified"),
            pair("n\nl.txt", "union"),
            pair("-dash.txt", "union"),
            pair("sp ace", "unspecified"),
        ]
    );
}

#[test]
fn a_dot_slash_path_is_matched_as_sent() {
    assert_eq!(
        read_attr_check(b"./a.txt\x00merge\x00union\x00", &paths(&["./a.txt"])).unwrap(),
        vec![pair("./a.txt", "union")]
    );
}

#[test]
fn fewer_triples_than_paths_are_refused_naming_the_output() {
    let err = read_attr_check(b"a.txt\x00merge\x00union\x00", &paths(&["a.txt", "b.dat"])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(
        err.stderr,
        "not a check-attr answer (not one path, merge, value triple per path sent): \"a.txt\\0merge\\0union\\0\""
    );
}

#[test]
fn more_triples_than_paths_are_refused() {
    assert!(read_attr_check(SEVEN, &paths(&["a.txt"])).is_err());
}

#[test]
fn a_path_differing_from_the_one_sent_is_refused() {
    let err = read_attr_check(b"./a.txt\x00merge\x00union\x00", &paths(&["a.txt"])).unwrap_err();
    assert_eq!(err.stderr, "not a check-attr answer (another path): \"./a.txt\\0merge\\0union\\0\"");
}

#[test]
fn paths_in_another_order_are_refused() {
    let err = read_attr_check(
        b"b.dat\x00merge\x00unset\x00a.txt\x00merge\x00union\x00",
        &paths(&["a.txt", "b.dat"]),
    )
    .unwrap_err();
    assert!(err.stderr.starts_with("not a check-attr answer (another path): "), "{}", err.stderr);
}

#[test]
fn an_attribute_other_than_merge_is_refused() {
    let err = read_attr_check(b"a.txt\x00diff\x00set\x00", &paths(&["a.txt"])).unwrap_err();
    assert_eq!(err.stderr, "not a check-attr answer (another attribute): \"a.txt\\0diff\\0set\\0\"");
}

#[test]
fn a_missing_final_nul_is_refused() {
    let err = read_attr_check(b"a.txt\x00merge\x00union", &paths(&["a.txt"])).unwrap_err();
    assert_eq!(err.stderr, "not a check-attr answer (no final NUL): \"a.txt\\0merge\\0union\"");
}

#[test]
fn empty_output_for_a_path_is_refused() {
    let err = read_attr_check(b"", &paths(&["a.txt"])).unwrap_err();
    assert_eq!(err.stderr, "not a check-attr answer (no final NUL): \"\"");
}

#[test]
fn an_empty_value_is_refused() {
    let err = read_attr_check(b"a.txt\x00merge\x00\x00", &paths(&["a.txt"])).unwrap_err();
    assert_eq!(err.stderr, "not a check-attr answer (an empty or non-UTF-8 value): \"a.txt\\0merge\\0\\0\"");
}

#[test]
fn a_non_utf8_value_is_refused() {
    assert!(read_attr_check(b"a.txt\x00merge\x00\xff\x00", &paths(&["a.txt"])).is_err());
}
