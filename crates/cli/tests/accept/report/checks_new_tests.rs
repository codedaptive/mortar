use super::{Checks, Line};

#[test]
#[ignore = "acceptance: make accept"]
fn new_checks_hold_no_lines() {
    assert_eq!(Checks::new().lines(), Vec::<Line>::new());
}
