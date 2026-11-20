//! The floor at below, equal, above, and a floor of 0.

use super::{floor_check, BelowFloor, SweepEstimate};

fn sweep() -> SweepEstimate {
    SweepEstimate {
        bytes: 123,
        reason: Some("not measured".to_string()),
    }
}

#[test]
fn below_is_refused_with_the_three_values() {
    assert_eq!(
        floor_check(9_999_999_999, 10_000_000_000, &sweep()),
        Err(BelowFloor {
            free: 9_999_999_999,
            floor: 10_000_000_000,
            sweep: SweepEstimate {
                bytes: 123,
                reason: Some("not measured".to_string()),
            },
        })
    );
}

#[test]
fn equal_is_allowed() {
    assert_eq!(floor_check(10_000_000_000, 10_000_000_000, &sweep()), Ok(()));
}

#[test]
fn above_is_allowed() {
    assert_eq!(floor_check(10_000_000_001, 10_000_000_000, &sweep()), Ok(()));
}

#[test]
fn floor_zero_allows_no_free_disk() {
    assert_eq!(floor_check(0, 0, &sweep()), Ok(()));
}
