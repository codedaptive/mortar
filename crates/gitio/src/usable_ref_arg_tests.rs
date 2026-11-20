use crate::usable_ref_arg::usable_ref_arg;

#[test]
fn full_refname_passes() {
    assert_eq!(usable_ref_arg("source ref", "refs/heads/main"), Ok(()));
}

#[test]
fn short_pattern_passes() {
    assert_eq!(usable_ref_arg("ref pattern", "main"), Ok(()));
}

#[test]
fn dash_inside_passes() {
    assert_eq!(usable_ref_arg("ref name", "refs/heads/a-b"), Ok(()));
}

#[test]
fn empty_is_refused_naming_what() {
    let err = usable_ref_arg("destination ref", "").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable destination ref: \"\"");
}

#[test]
fn leading_dash_is_refused() {
    let err = usable_ref_arg("ref pattern", "-v").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable ref pattern: \"-v\"");
}

#[test]
fn colon_is_refused() {
    let err = usable_ref_arg("ref name", "refs/heads/a:refs/heads/b").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable ref name: \"refs/heads/a:refs/heads/b\"");
}

#[test]
fn nul_is_refused_naming_what() {
    let err = usable_ref_arg("source ref", "refs/heads/m\0").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "source ref holds a NUL: \"refs/heads/m\\0\"");
}
