//! Whether a logged `mr`/`mortar` call asks for help or the version, which reads no repository and changes nothing
//! (step 4's check 4). It mirrors mr's own reading in crates/cli/src/help/scan_help_words.rs and
//! crates/cli/src/version/version_asked.rs, which the accept target cannot call (the cli crate exports only
//! main_entry), so a change there must be made here too.

/// `args` is the logged argv, args[0] the program. True when the words after it, up to the `--` that ends the
/// globals, hold `--help` or `-h` anywhere, or their first word that is no global flag is `help`. `--json` is a
/// global flag; `--key` and `--actor` take the next word as their value unless it is `--help` or `-h` (so
/// `--key --` keeps that `--` as the value); `--key=<v>` and `--actor=<v>` are one word. Any `--help` counts, so
/// `mr nosuch --help`, which mr answers with a usage error and no change, counts too. Also true when the words
/// after the program are exactly `--version`, with at most one `--json` before or after it. Else false.
pub fn help_or_version_call(args: &[String]) -> bool {
    let argv: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    help_asked(&argv) || matches!(argv.as_slice(), ["--version"] | ["--version", "--json"] | ["--json", "--version"])
}

/// scan_help_words's reading: `--help`/`-h` before the `--` that ends the globals, or a first other word `help`.
fn help_asked(argv: &[&str]) -> bool {
    let mut read = argv.iter().copied().peekable();
    let mut first_word = None;
    while let Some(word) = read.next() {
        match word {
            "--" => break,
            "--json" => {}
            "--key" | "--actor" => {
                read.next_if(|value| !matches!(*value, "--help" | "-h"));
            }
            "--help" | "-h" => return true,
            _ if word.starts_with("--key=") || word.starts_with("--actor=") => {}
            _ => {
                first_word.get_or_insert(word);
            }
        }
    }
    first_word == Some("help")
}
