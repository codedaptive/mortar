use super::{Checks, Line};

#[test]
fn new_checks_hold_no_lines() {
    assert_eq!(Checks::new().lines(), Vec::<Line>::new());
}
