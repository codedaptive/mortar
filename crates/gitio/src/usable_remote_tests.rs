use crate::usable_remote::usable_remote;

#[test]
fn configured_name_passes() {
    assert_eq!(usable_remote("origin"), Ok(()));
}

#[test]
fn relative_path_passes() {
    assert_eq!(usable_remote("../remote.git"), Ok(()));
}

#[test]
fn url_passes() {
    assert_eq!(usable_remote("ssh://host.invalid/r.git"), Ok(()));
}

#[test]
fn dash_inside_passes() {
    assert_eq!(usable_remote("my-remote"), Ok(()));
}

#[test]
fn empty_is_refused() {
    let err = usable_remote("").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable remote: \"\"");
}

#[test]
fn leading_dash_is_refused_naming_it() {
    let err = usable_remote("-oops").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable remote: \"-oops\"");
}

#[test]
fn nul_is_refused_naming_it() {
    let err = usable_remote("orig\0in").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "remote holds a NUL: \"orig\\0in\"");
}
