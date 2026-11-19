//! scan_help_words: the global flags and other words of a command line, read for help.

use super::HelpWords;

/// Reads `argv` (without the program name) up to the `--` that ends the globals, with parse_args's global rules:
/// `--json` sets json (a second one is no error here, so a line parse_args refuses for its globals still gets help);
/// `--key` and `--actor` take the next word as their value, unless it is `--help` or `-h` (so `--key -- help` reads
/// `--` as the value, not as the end of the globals); `--key=<v>` and `--actor=<v>` are one word.
/// A `--help` or `-h` before that `--` always counts, even where a flag's value would stand: it sets flagged and is
/// dropped, not taken as a value. The other words, in order, are the words.
pub fn scan_help_words(argv: &[String]) -> HelpWords<'_> {
    let mut scan = HelpWords { json: false, flagged: false, words: Vec::new() };
    let mut read = argv.iter().map(String::as_str).peekable();
    while let Some(word) = read.next() {
        match word {
            "--" => break,
            "--json" => scan.json = true,
            "--key" | "--actor" => {
                read.next_if(|value| !matches!(*value, "--help" | "-h"));
            }
            "--help" | "-h" => scan.flagged = true,
            _ if word.starts_with("--key=") || word.starts_with("--actor=") => {}
            _ => scan.words.push(word),
        }
    }
    scan
}
