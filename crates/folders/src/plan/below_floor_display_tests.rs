//! The below-floor refusal's exact text, with and without the sweep's reason.

use super::{BelowFloor, SweepEstimate};

#[test]
fn names_free_floor_and_sweep_bytes() {
    let err = BelowFloor {
        free: 4_000_000_000,
        floor: 10_000_000_000,
        sweep: SweepEstimate {
            bytes: 7_500_000_000,
            reason: None,
        },
    };
    assert_eq!(
        err.to_string(),
        "free disk 4000000000 bytes is under the floor of 10000000000 bytes; a sweep would free 7500000000 bytes"
    );
}

#[test]
fn adds_the_sweep_reason_when_given() {
    let err = BelowFloor {
        free: 1,
        floor: 2,
        sweep: SweepEstimate {
            bytes: 0,
            reason: Some("not measured".to_string()),
        },
    };
    assert_eq!(
        err.to_string(),
        "free disk 1 bytes is under the floor of 2 bytes; a sweep would free 0 bytes (not measured)"
    );
}

#[test]
fn is_a_std_error_with_no_source() {
    let err: Box<dyn std::error::Error> = Box::new(BelowFloor {
        free: 1,
        floor: 2,
        sweep: SweepEstimate { bytes: 3, reason: None },
    });
    assert!(err.source().is_none());
}
