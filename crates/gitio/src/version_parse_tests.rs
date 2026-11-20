use crate::Version;

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
}

#[test]
fn parses_plain_release() {
    assert_eq!(Version::parse(b"git version 2.56.0\n"), Ok(v(2, 56, 0)));
}

#[test]
fn parses_apple_build() {
    assert_eq!(Version::parse(b"git version 2.39.3 (Apple Git-146)\n"), Ok(v(2, 39, 3)));
}

#[test]
fn parses_windows_build() {
    assert_eq!(Version::parse(b"git version 2.45.1.windows.1"), Ok(v(2, 45, 1)));
}

#[test]
fn missing_patch_is_zero() {
    assert_eq!(Version::parse(b"git version 2.40"), Ok(v(2, 40, 0)));
}

#[test]
fn refuses_garbage_empty_non_utf8_and_missing_minor() {
    let bad: [&[u8]; 8] = [
        b"git version +2.40.0",
        b"git version 2.+40.0",
        b"hello world",
        b"",
        b"git version \xff2.40.0",
        b"git version 2",
        b"git version",
        b"version 2.40.0",
    ];
    for text in bad {
        assert!(Version::parse(text).is_err(), "{text:?} accepted");
    }
}

#[test]
fn refusal_names_the_text_found() {
    let err = Version::parse(b"not git at all").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("not git at all"), "{}", err.stderr);
}
