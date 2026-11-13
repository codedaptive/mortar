//! Decoding lock record text: a literal, the round trip over many records, and malformed text refused.

use super::{decode_lock_record, encode_lock_record, LockError, LockRecord};

#[test]
fn decodes_literal() {
    assert_eq!(
        decode_lock_record("pid=4242\nhost=build-mac\nkey=a=b\nsince=1763030400\n").unwrap(),
        LockRecord {
            pid: 4242,
            host: "build-mac".to_string(),
            key: "a=b".to_string(),
            since: 1763030400,
        }
    );
}

#[test]
fn round_trips_many_records() {
    let pids = [0, 1, 7, 4242, 65536, u32::MAX - 1, u32::MAX];
    let sinces = [0, 1, 1763030400, u64::from(u32::MAX) + 1, u64::MAX - 1, u64::MAX];
    let hosts = ["h", "build-mac.local", "hôte-ünïcödé", "主机", "a=b"];
    let keys = ["k", "join:zone-a", "a=b", "==", "x=", "op key with spaces", "ключ=значение"];
    let mut count = 0;
    for pid in pids {
        for since in sinces {
            for host in hosts {
                for key in keys {
                    let r = LockRecord {
                        pid,
                        host: host.to_string(),
                        key: key.to_string(),
                        since,
                    };
                    let text = encode_lock_record(&r).unwrap();
                    assert_eq!(decode_lock_record(&text).unwrap(), r, "text {text:?}");
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 1470);
}

#[test]
fn refuses_malformed_text() {
    let bad = [
        ("empty", ""),
        ("missing field", "pid=1\nhost=h\nsince=2\n"),
        ("duplicate field", "pid=1\npid=1\nhost=h\nkey=k\n"),
        ("wrong order", "host=h\npid=1\nkey=k\nsince=2\n"),
        ("extra line", "pid=1\nhost=h\nkey=k\nsince=2\nmore=x\n"),
        ("trailing blank line", "pid=1\nhost=h\nkey=k\nsince=2\n\n"),
        ("missing final newline", "pid=1\nhost=h\nkey=k\nsince=2"),
        ("non-numeric pid", "pid=abc\nhost=h\nkey=k\nsince=2\n"),
        ("negative pid", "pid=-1\nhost=h\nkey=k\nsince=2\n"),
        ("plus-signed pid", "pid=+1\nhost=h\nkey=k\nsince=2\n"),
        ("overflow pid", "pid=4294967296\nhost=h\nkey=k\nsince=2\n"),
        ("spaced pid", "pid= 1\nhost=h\nkey=k\nsince=2\n"),
        ("empty pid", "pid=\nhost=h\nkey=k\nsince=2\n"),
        ("overflow since", "pid=1\nhost=h\nkey=k\nsince=18446744073709551616\n"),
        ("negative since", "pid=1\nhost=h\nkey=k\nsince=-2\n"),
        ("empty host", "pid=1\nhost=\nkey=k\nsince=2\n"),
        ("empty key", "pid=1\nhost=h\nkey=\nsince=2\n"),
        ("line with no '='", "pid=1\nhost\nkey=k\nsince=2\n"),
        ("carriage return", "pid=1\nhost=h\r\nkey=k\nsince=2\n"),
    ];
    for (what, text) in bad {
        assert!(
            matches!(decode_lock_record(text), Err(LockError::Malformed { path: None, .. })),
            "{what}: {text:?} was not refused"
        );
    }
}
