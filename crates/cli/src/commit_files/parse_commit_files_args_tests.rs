//! parse_commit_files_args: paths with at most one --message text before or after them are the answer; a --message
//! with no text, an empty or blank text, or given twice, an unknown option and no path are refused, the first reason
//! in line order.

use std::path::PathBuf;

use super::parse_commit_files_args;

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

fn read(paths: &[&str], message: Option<&str>) -> Result<(Vec<PathBuf>, Option<String>), String> {
    Ok((paths.iter().map(PathBuf::from).collect(), message.map(str::to_string)))
}

fn refused(reason: &str) -> Result<(Vec<PathBuf>, Option<String>), String> {
    Err(reason.to_string())
}

#[test]
fn the_paths_and_the_message_are_read_or_the_first_reason_is_given() {
    for (line, expected) in [
        (&["src/a.rs"][..], read(&["src/a.rs"], None)),
        (&["a", "b/c", "d e"], read(&["a", "b/c", "d e"], None)),
        (&["a", "--message", "Subject\n\nBody."], read(&["a"], Some("Subject\n\nBody."))),
        (&["--message", " kept as given ", "a", "b"], read(&["a", "b"], Some(" kept as given "))),
        (&["a", "--message=x: y", "b"], read(&["a", "b"], Some("x: y"))),
        (&["--message", "--json", "a"], read(&["a"], Some("--json"))),
        (&["a", "--message"], refused("--message needs a text")),
        (&["a", "--message", "one", "--message=two"], refused("--message is given twice")),
        (&["a", "--message", "one", "--message"], refused("--message needs a text")),
        (&["a", "--message", ""], refused("--message needs a text that is not empty or only whitespace")),
        (&["a", "--message="], refused("--message needs a text that is not empty or only whitespace")),
        (&["--message", " \n\t", "a"], refused("--message needs a text that is not empty or only whitespace")),
        (&["a", "--force"], refused("unknown option \"--force\"")),
        (&["-", "a"], refused("unknown option \"-\"")),
        (&["--force", "--message"], refused("unknown option \"--force\"")),
        (&["--message", " ", "--force"], refused("--message needs a text that is not empty or only whitespace")),
        (&[], refused("commit-files needs at least one path")),
        (&["--message", "text"], refused("commit-files needs at least one path")),
    ] {
        assert_eq!(parse_commit_files_args(&words(line)), expected, "{line:?}");
    }
}
