use super::{Checks, Line};

#[test]
#[ignore = "acceptance: make accept"]
fn check_returns_ok_and_lines_keep_the_order_recorded() {
    let mut checks = Checks::new();
    assert!(checks.check("ran init", true, "exit 0"));
    checks.log("op 1 join intent");
    assert!(!checks.check("ran log", false, "exit 2"));
    assert_eq!(
        checks.lines(),
        vec![
            Line::Check { ok: true, did: "ran init".to_string(), saw: "exit 0".to_string() },
            Line::Log("op 1 join intent".to_string()),
            Line::Check { ok: false, did: "ran log".to_string(), saw: "exit 2".to_string() },
        ]
    );
}
