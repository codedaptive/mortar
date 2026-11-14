//! ref_format_verdict: a well-formed name passes; a malformed one is BadMove naming it, with the fix.

use super::ref_format_verdict;
use crate::store::OplogError;

#[test]
fn a_well_formed_name_passes() {
    assert!(ref_format_verdict("refs/heads/main", true).is_ok());
}

#[test]
fn a_malformed_name_is_refused_by_name() {
    match ref_format_verdict("refs/heads/a..b", false) {
        Err(OplogError::BadMove { refname, reason }) => assert_eq!(
            (refname.as_str(), reason.as_str()),
            (
                "refs/heads/a..b",
                "is not a valid ref name (git check-ref-format refused it)"
            )
        ),
        other => panic!("expected BadMove, got {other:?}"),
    }
}

#[test]
fn the_refusal_reads_with_the_ref_and_the_fix() {
    assert_eq!(
        ref_format_verdict("refs/heads/a..b", false).unwrap_err().to_string(),
        "op cannot be applied: ref \"refs/heads/a..b\" is not a valid ref name (git check-ref-format refused \
         it); fix the op's moves and run again"
    );
}
