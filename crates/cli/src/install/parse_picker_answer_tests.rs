//! parse_picker_answer: none, all, or a comma list of detected names in table order.

use super::{parse_picker_answer, HarnessSpec, HARNESSES};

fn claude_and_cursor() -> Vec<&'static HarnessSpec> {
    vec![&HARNESSES[0], &HARNESSES[2]]
}

fn picked(answer: &str) -> Vec<&'static str> {
    parse_picker_answer(answer, &claude_and_cursor()).expect("a good answer").iter().map(|h| h.name).collect()
}

#[test]
fn an_empty_answer_picks_none() {
    assert!(picked("").is_empty());
    assert!(picked("  \n").is_empty());
}

#[test]
fn all_picks_every_detected_one() {
    assert_eq!(picked("all"), vec!["claude", "cursor"]);
    assert_eq!(picked(" all\n"), vec!["claude", "cursor"]);
}

#[test]
fn a_list_is_trimmed_kept_once_and_in_table_order() {
    assert_eq!(picked("cursor"), vec!["cursor"]);
    assert_eq!(picked(" cursor , claude\n"), vec!["claude", "cursor"]);
    assert_eq!(picked("claude,,claude,"), vec!["claude"]);
}

#[test]
fn a_name_not_detected_is_refused_with_the_detected_list() {
    assert_eq!(
        parse_picker_answer("claude, codex", &claude_and_cursor()),
        Err("\"codex\" is not a detected harness (detected: claude, cursor)".to_string())
    );
    assert_eq!(
        parse_picker_answer("vim", &claude_and_cursor()),
        Err("\"vim\" is not a detected harness (detected: claude, cursor)".to_string())
    );
}
