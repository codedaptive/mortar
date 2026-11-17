use super::{read_reply, setup_step, test_ran};
use crate::report::{Checks, Line};

#[test]
fn the_outcome_asked_for_holds_and_lets_the_set_up_go_on() {
    let mut checks = Checks::new();
    let reply = read_reply(&test_ran(Some(0), r#"{"outcome":"made","lines":[]}"#));
    assert_eq!(setup_step(&mut checks, "set-up: mr folder request", &reply, "made"), Ok(()));
    assert_eq!(
        checks.lines(),
        vec![Line::Check { ok: true, did: "set-up: mr folder request".to_string(), saw: "exit 0; outcome made".to_string() }]
    );
}

#[test]
fn another_outcome_or_exit_fails_the_line_and_names_it() {
    let mut checks = Checks::new();
    let refused = read_reply(&test_ran(Some(1), r#"{"outcome":"refused","lines":["zone branch zone/a does not exist"]}"#));
    assert_eq!(
        setup_step(&mut checks, "set-up: mr folder zone a", &refused, "made"),
        Err("set-up: mr folder zone a: exit 1; outcome refused; zone branch zone/a does not exist".to_string())
    );
    let wrong = read_reply(&test_ran(Some(0), r#"{"outcome":"unchanged","lines":[]}"#));
    assert!(setup_step(&mut checks, "set-up: mr save", &wrong, "saved").is_err());
    let lines = checks.lines();
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|line| matches!(line, Line::Check { ok: false, .. })));
}
