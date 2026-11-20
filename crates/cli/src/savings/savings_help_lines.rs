//! The lines mr help adds about tokens saved.

const LINES: [&str; 8] = [
    "",
    "Tokens saved: an answer that replaces reading git output opens with a \"[mr] tokens saved\" line,",
    "and its --json carries a \"savings\" object. The number is an upper bound: it counts as if the whole",
    "git output the answer replaces would have been read.",
    "{\"savings_line\": false} in $XDG_CONFIG_HOME/mortar/config.json (else ~/.config/mortar/config.json)",
    "turns the line off; --json keeps \"savings\".",
    "Each such answer also appends a line to $XDG_STATE_HOME/mortar/savings.jsonl",
    "(else ~/.local/state/mortar/savings.jsonl).",
];

/// An empty line, then what the "[mr] tokens saved" line and the --json "savings" object are, that the number is
/// an upper bound, the switch that turns the line off and its file, and the savings log's file.
pub fn savings_help_lines() -> Vec<String> {
    LINES.iter().map(|line| line.to_string()).collect()
}
