//! program_help_lines: the usage line, the four groups' headings in order, one row per term padded to the longest
//! term, the thirteen lines about exit codes, then the eight lines about tokens saved.
//! Unit test: the help groups are pure data.
//! The longest term is bisect's, "bisect <last pass> <failed> --timeout <s> -- <command>..." (57 characters), so
//! every term is padded to 57: "status" (6) with 51 spaces, "install, uninstall" (18) with 39.

use super::program_help_lines;

const STATUS_ROW: &str = "  status                                                     where you are, what you changed, \
                          whether your zone moved, anything conflicted, and the next step";
const INSTALL_ROW: &str = "  install, uninstall                                         place mortar and mr and offer \
                           skill wiring; remove what install placed";
const BISECT_ROW: &str = "  bisect <last pass> <failed> --timeout <s> -- <command>...  find the landing that broke a test";

#[test]
fn the_first_line_is_the_usage() {
    assert_eq!(program_help_lines()[0], "mr <verb> [args] [--json] [--key <operation key>] [--actor <name>]");
}

#[test]
fn the_headings_follow_an_empty_line_in_order() {
    let lines = program_help_lines();
    let headings = lines[..39]
        .windows(2)
        .filter(|pair| pair[0].is_empty())
        .map(|pair| pair[1].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        headings,
        vec![
            "Agent verbs (run in your request folder)",
            "Folder and upkeep verbs (for the harness)",
            "Landing verbs (for the landing script)",
            "Setup",
        ]
    );
}

#[test]
fn the_status_row_is_padded_to_the_longest_term() {
    assert_eq!(program_help_lines()[3], STATUS_ROW);
}

#[test]
fn the_longest_term_has_two_spaces_before_its_line() {
    assert!(program_help_lines().iter().any(|line| line == BISECT_ROW));
}

#[test]
fn install_and_uninstall_share_one_row() {
    let lines = program_help_lines();
    assert_eq!(lines.iter().filter(|line| *line == INSTALL_ROW).count(), 1);
    assert_eq!(lines.iter().filter(|line| line.contains("uninstall")).count(), 1);
}

#[test]
fn backup_and_restore_backup_share_one_row() {
    let lines = program_help_lines();
    assert_eq!(lines.iter().filter(|line| line.contains("restore-backup")).count(), 1);
}

#[test]
fn there_are_thirty_six_rows_and_no_trailing_spaces() {
    // 6 agent rows, 8 folder, 13 landing (backup shares one), 3 setup (install shares one): 30. With the usage line
    // and an empty line and a heading per group, 39 lines; then the 13 lines about exit codes, 6 of them rows: 52;
    // then the 8 lines about tokens saved: 60.
    let lines = program_help_lines();
    assert_eq!(lines.iter().filter(|line| line.starts_with("  ")).count(), 36);
    assert_eq!(lines.len(), 60);
    assert!(lines.iter().all(|line| !line.ends_with(' ')), "{lines:#?}");
}

#[test]
fn the_tokens_saved_lines_follow_the_exit_codes() {
    assert_eq!(
        program_help_lines()[52..],
        [
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
