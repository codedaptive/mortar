//! savings_help_lines: the exact lines mr help adds after its groups.

use super::savings_help_lines;

#[test]
fn the_exact_lines() {
    assert_eq!(
        savings_help_lines(),
        vec![
            "",
            "Tokens saved: an answer that replaces reading git output opens with a \"[mr] tokens saved\" line,",
            "and its --json carries a \"savings\" object. The number is an upper bound: it counts as if the whole",
            "git output the answer replaces would have been read.",
            "{\"savings_line\": false} in $XDG_CONFIG_HOME/mortar/config.json (else ~/.config/mortar/config.json)",
            "turns the line off; --json keeps \"savings\".",
            "Each such answer also appends a line to $XDG_STATE_HOME/mortar/savings.jsonl",
            "(else ~/.local/state/mortar/savings.jsonl).",
        ]
    );
}

#[test]
fn every_line_is_short() {
    for line in savings_help_lines() {
        assert!(line.chars().count() <= 110, "{line}");
    }
}
