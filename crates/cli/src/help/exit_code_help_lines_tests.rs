//! exit_code_help_lines: an empty line, the heading, the six codes 0 to 5 with their words (CLI-SPEC §4: codes 3 and
//! up listed in --help), then the lines on which code wins.
//! Unit test: the block is pure data.

use super::exit_code_help_lines;

#[test]
fn the_block_is_the_heading_the_six_codes_and_which_code_wins() {
    assert_eq!(
        exit_code_help_lines(),
        [
            "",
            "Exit codes",
            "  0  done: the command did its work, or found it already done",
            "  1  conflict: the work conflicted; the report lists the conflicted files",
            "  2  usage: the command line could not be read; the report gives the verb's usage",
            "  3  refused: the report names who owns the fix and the next step",
            "  4  busy: a lock is held or a ref moved mid-run; repeat it later, or first take the step the report names",
            "  5  failed: git or the disk failed; the report names the next step",
            "Each run answers one code, from the first check that stops it: the command line is read first (2), then",
            "git's version, the folder, the actor and the config (2, 3 or 5); then the verb checks its own arguments (2)",
            "and what it acts on (3 or 5) before it takes any lock (4); the work itself ends in any code but 2.",
        ]
    );
}
