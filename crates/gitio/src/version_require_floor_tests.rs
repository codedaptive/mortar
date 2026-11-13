use crate::Version;

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
}

#[test]
fn accepts_the_floor_and_above() {
    assert_eq!(v(2, 40, 0).require_floor(), Ok(v(2, 40, 0)));
    assert_eq!(v(2, 56, 0).require_floor(), Ok(v(2, 56, 0)));
    assert_eq!(v(3, 0, 0).require_floor(), Ok(v(3, 0, 0)));
}

#[test]
fn refuses_apple_git_naming_the_version_found() {
    let parsed = Version::parse(b"git version 2.39.3 (Apple Git-146)\n").unwrap();
    let err = parsed.require_floor().unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "git 2.39.3 found; mortar needs git 2.40 or newer");
}

#[test]
fn refuses_anything_under_2_40() {
    for old in [v(2, 39, 99), v(2, 38, 0), v(1, 99, 0)] {
        assert!(old.require_floor().is_err(), "{old:?} accepted");
    }
}
