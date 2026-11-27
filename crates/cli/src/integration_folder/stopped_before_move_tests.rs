//! stopped_before_move: exit 1 "folder-not-moved" led by the verb's done line, the failed read and that the folder
//! was not moved, the verb's stop line, and the failed read's next ending with the repeat.

use super::stopped_before_move;
use crate::report::{failure_report, ExitCode, Report};

#[test]
fn a_failed_read_before_the_move_is_led_by_the_done_line_and_ends_with_the_repeat() {
    let rows = [
        (
            "landed change C: integration is at T",
            failure_report("land", "the op log could not be read"),
            "mr land X --actor <name>",
            Report {
                verb: "land".to_string(),
                exit: ExitCode::Refused,
                outcome: "folder-not-moved".to_string(),
                lines: vec![
                    "landed change C: integration is at T".to_string(),
                    "the op log could not be read, so the integration folder was not moved".to_string(),
                    "land stopped here".to_string(),
                ],
                owner: None,
                next: "repeat the same command: mr land X --actor <name>".to_string(),
                fields: Vec::new(),
            },
        ),
        (
            "backed out change C",
            failure_report("back-out", "git could not read refs/heads/integration"),
            "mr back-out --actor <name>",
            Report {
                verb: "back-out".to_string(),
                exit: ExitCode::Refused,
                outcome: "folder-not-moved".to_string(),
                lines: vec![
                    "backed out change C".to_string(),
                    "git could not read refs/heads/integration, so the integration folder was not moved".to_string(),
                    "back-out stopped here".to_string(),
                ],
                owner: None,
                next: "repeat the same command: mr back-out --actor <name>".to_string(),
                fields: Vec::new(),
            },
        ),
    ];
    for (done, error, repeat, expected) in rows {
        assert_eq!(stopped_before_move(done, error, repeat), expected, "done line: {done}");
    }
}
