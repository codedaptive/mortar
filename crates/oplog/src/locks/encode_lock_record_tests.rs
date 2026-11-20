//! Lock record text, and the records refused.

use super::{encode_lock_record, LockError, LockRecord};

fn record(host: &str, key: &str) -> LockRecord {
    LockRecord {
        pid: 4242,
        host: host.to_string(),
        key: key.to_string(),
        since: 1763030400,
    }
}

#[test]
fn encodes_four_lines() {
    assert_eq!(
        encode_lock_record(&record("build-mac", "join:zone-a")).unwrap(),
        "pid=4242\nhost=build-mac\nkey=join:zone-a\nsince=1763030400\n"
    );
}

#[test]
fn key_may_hold_equals() {
    assert_eq!(
        encode_lock_record(&record("h", "a=b=c")).unwrap(),
        "pid=4242\nhost=h\nkey=a=b=c\nsince=1763030400\n"
    );
}

#[test]
fn refuses_empty_or_line_breaking_host_and_key() {
    let bad = [
        record("", "k"),
        record("h", ""),
        record("h\nx", "k"),
        record("h\rx", "k"),
        record("h", "k\n"),
        record("h", "\rk"),
    ];
    for r in bad {
        assert!(
            matches!(encode_lock_record(&r), Err(LockError::BadRecord(_))),
            "{r:?} was not refused"
        );
    }
}
