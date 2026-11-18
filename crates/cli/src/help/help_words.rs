//! help_words: what scan_help_words reads from a command line's globals (SPEC-cli 08, "The calls").
//! A bare type (S-7); scan_help_words_tests exercises it.

/// `json`: `--json` came before the `--` that ends the globals. `flagged`: `--help` or `-h` came there. `words`: the
/// other words before that `--`, in order, with the global flags and their values dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpWords<'a> {
    pub json: bool,
    pub flagged: bool,
    pub words: Vec<&'a str>,
}
