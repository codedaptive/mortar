//! op_id_from_parts: `<nanos>-<pid>-<counter>` in lowercase hex; distinct parts give distinct ids.

use std::collections::HashSet;

use super::op_id_from_parts;

#[test]
fn small_parts() {
    assert_eq!(op_id_from_parts(1, 2, 3), "1-2-3");
}

#[test]
fn parts_are_lowercase_hex() {
    assert_eq!(op_id_from_parts(255, 4096, 10), "ff-1000-a");
}

#[test]
fn zero_parts() {
    assert_eq!(op_id_from_parts(0, 0, 0), "0-0-0");
}

#[test]
fn a_realistic_clock_reading() {
    assert_eq!(
        op_id_from_parts(1_763_030_400_123_456_789, 48_213, 7),
        "18778aced792cd15-bc55-7"
    );
}

#[test]
fn largest_parts() {
    assert_eq!(
        op_id_from_parts(u128::MAX, u32::MAX, u64::MAX),
        "ffffffffffffffffffffffffffffffff-ffffffff-ffffffffffffffff"
    );
}

#[test]
fn distinct_parts_give_distinct_ids() {
    let parts: [(u128, u32, u64); 8] = [
        (1, 23, 4),
        (12, 3, 4),
        (1, 2, 34),
        (12, 34, 0),
        (0, 1, 234),
        (123, 4, 0),
        (1, 234, 0),
        (0, 0, 1234),
    ];
    let ids: HashSet<String> = parts
        .iter()
        .map(|&(n, p, c)| op_id_from_parts(n, p, c))
        .collect();
    assert_eq!(ids.len(), 8);
}
