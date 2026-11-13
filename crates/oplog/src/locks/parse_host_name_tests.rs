//! Host names from `hostname` output.

use super::{parse_host_name, LockError};

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(parse_host_name(b"build-mac.local\n").unwrap(), "build-mac.local");
    assert_eq!(parse_host_name(b"  \tbox-1 \r\n").unwrap(), "box-1");
}

#[test]
fn accepts_unicode() {
    assert_eq!(parse_host_name("hôte\n".as_bytes()).unwrap(), "hôte");
}

#[test]
fn refuses_bad_output() {
    let bad: [&[u8]; 6] = [b"", b" \n", b"two words\n", b"a\tb", b"a\x07b", b"\xff\xfe\n"];
    for out in bad {
        assert!(
            matches!(parse_host_name(out), Err(LockError::HostUnknown(_))),
            "{out:?} was not refused"
        );
    }
}
