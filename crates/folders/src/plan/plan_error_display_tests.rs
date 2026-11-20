//! The plan refusal's text, and its use as a std error.

use super::PlanError;

#[test]
fn display_prefixes_the_reason() {
    let err = PlanError {
        reason: "zone name \"\" is empty".to_string(),
    };
    assert_eq!(err.to_string(), "folder plan: zone name \"\" is empty");
}

#[test]
fn is_a_std_error_with_no_source() {
    let err: Box<dyn std::error::Error> = Box::new(PlanError {
        reason: "x".to_string(),
    });
    assert_eq!(err.to_string(), "folder plan: x");
    assert!(err.source().is_none());
}
