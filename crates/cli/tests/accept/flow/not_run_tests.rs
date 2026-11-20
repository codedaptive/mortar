use super::not_run;
use crate::report::Checks;
use crate::stage::failed_line as fail;

#[test]
fn the_checks_from_the_one_given_to_the_last_fail_in_order() {
    let mut checks = Checks::new();
    not_run(&mut checks, 13, "check 12 failed");
    assert_eq!(
        checks.lines(),
        vec![
            fail("13. published", "not run: check 12 failed"),
            fail("14. the other zone replayed", "not run: check 12 failed"),
            fail("15. plain git throughout", "not run: check 12 failed"),
        ]
    );
}

#[test]
fn from_the_first_lists_all_fifteen() {
    let mut checks = Checks::new();
    not_run(&mut checks, 1, "the set-up failed");
    let lines = checks.lines();
    assert_eq!(lines.len(), 15);
    assert_eq!(lines[0], fail("1. requests made", "not run: the set-up failed"));
    assert_eq!(lines[7], fail("8. checked for merging", "not run: the set-up failed"));
}
