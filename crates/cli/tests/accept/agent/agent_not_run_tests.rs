use super::agent_not_run;
use crate::report::Checks;
use crate::stage::failed_line as fail;

#[test]
fn the_conflict_line_and_all_four_checks_fail_with_the_reason() {
    let mut checks = Checks::new();
    agent_not_run(&mut checks, "the set-up failed");
    let lines = checks.lines();
    assert_eq!(lines.len(), 5);
    assert_eq!(
        lines[0],
        fail(
            "the conflict: the drive's mr join in rc came before the agent's mr join, and the agent ran mr resolve",
            "not run: the set-up failed",
        )
    );
    assert_eq!(
        lines[4],
        fail(
            "4. every mr or mortar call of the agent's in mr.log names one of the six verbs",
            "not run: the set-up failed",
        )
    );
}
