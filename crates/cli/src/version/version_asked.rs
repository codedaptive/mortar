//! version_asked: Whether the command line asks for `--version`.

/// Some(json) when `argv` holds `--version` once and nothing else but at most one `--json`, in any order; else None.
pub fn version_asked(argv: &[String]) -> Option<bool> {
    let words: Vec<&str> = argv.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["--version"] => Some(false),
        ["--version", "--json"] | ["--json", "--version"] => Some(true),
        _ => None,
    }
}
