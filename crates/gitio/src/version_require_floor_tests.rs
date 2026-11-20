use crate::Version;

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
}

#[test]
fn accepts_the_floor_and_above() {
    assert_eq!(v(2, 46, 0).require_floor(), Ok(v(2, 46, 0)));
    assert_eq!(v(2, 56, 0).require_floor(), Ok(v(2, 56, 0)));
    assert_eq!(v(3, 0, 0).require_floor(), Ok(v(3, 0, 0)));
}

#[test]
fn refuses_apple_git_naming_the_version_found() {
    let parsed = Version::parse(b"git version 2.45.4 (Apple Git-150)\n").unwrap();
    let err = parsed.require_floor().unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "git 2.45.4 found; mortar needs git 2.46 or newer");
}

#[test]
fn refuses_anything_under_2_46() {
    for (old, stderr) in [
        (v(2, 45, 99), "git 2.45.99 found; mortar needs git 2.46 or newer"),
        (v(2, 45, 0), "git 2.45.0 found; mortar needs git 2.46 or newer"),
        (v(1, 99, 0), "git 1.99.0 found; mortar needs git 2.46 or newer"),
    ] {
        let err = old.require_floor().expect_err("old version must be refused");
        assert_eq!(err.code, None);
        assert!(err.args.is_empty());
        assert_eq!(err.stderr, stderr);
    }
}
