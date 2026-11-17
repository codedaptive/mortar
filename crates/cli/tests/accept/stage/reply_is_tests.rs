use super::{read_reply, test_ran};

#[test]
fn the_exit_and_the_outcome_must_both_match() {
    let landed = read_reply(&test_ran(Some(0), r#"{"outcome":"landed"}"#));
    assert!(landed.is(0, "landed"));
    assert!(!landed.is(0, "busy"));
    assert!(!landed.is(3, "landed"));
}

#[test]
fn a_run_with_no_exit_code_never_matches() {
    assert!(!read_reply(&test_ran(None, r#"{"outcome":"landed"}"#)).is(0, "landed"));
}
