use super::kill_not_run;
use crate::report::Checks;
use crate::stage::failed_line as fail;

#[test]
fn from_the_first_case_lists_every_check_of_every_case() {
    let mut checks = Checks::new();
    kill_not_run(&mut checks, 0, "the set-up failed");
    let lines = checks.lines();
    // Cases 1, 2, 3 before, 3 after and 4 have 5, 5, 4, 4 and 5 own checks, and five consistency items each.
    assert_eq!(lines.len(), 48);
    assert_eq!(
        lines[0],
        fail(
            "1. mr join in r1 (a fast-forward), killed before its ref transaction: the op log's open intent is that join",
            "not run: the set-up failed",
        )
    );
    assert_eq!(
        lines[10],
        fail(
            "2. mr join in r2 (a rebase), killed after its ref transaction: the op log's open intent is that join",
            "not run: the set-up failed",
        )
    );
    assert_eq!(
        lines[47],
        fail("4. consistent: repeating the finished command once more moves nothing", "not run: the set-up failed")
    );
}

#[test]
fn from_a_later_case_starts_there() {
    let mut checks = Checks::new();
    kill_not_run(&mut checks, 4, "case 3 after failed");
    let lines = checks.lines();
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[2], fail("4. mr publish again: published", "not run: case 3 after failed"));
}
