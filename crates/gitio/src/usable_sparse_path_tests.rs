use crate::usable_sparse_path::usable_sparse_path;

#[test]
fn plain_spaced_dashed_and_quoted_paths_pass() {
    for path in ["d2", "sp ace", "-dash", "back\\sl", "q\"uote", "ta\tb", "café", "a/b/c"] {
        assert_eq!(usable_sparse_path(path), Ok(()), "{path:?}");
    }
}

#[test]
fn carriage_return_passes() {
    assert_eq!(usable_sparse_path("cr\rx"), Ok(()));
}

#[test]
fn empty_path_is_refused() {
    let err = usable_sparse_path("").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable sparse path (empty): \"\"");
}

#[test]
fn nul_is_refused_naming_the_path() {
    let err = usable_sparse_path("a\0b").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "sparse path holds a NUL: \"a\\0b\"");
}

#[test]
fn line_break_is_refused_naming_the_path() {
    let err = usable_sparse_path("n\nl").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "sparse path holds a line break: \"n\\nl\"");
}

#[test]
fn trailing_line_break_is_refused() {
    assert!(usable_sparse_path("d1\n").is_err());
}
