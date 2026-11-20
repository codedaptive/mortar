//! parse_restore_backup_args: exactly one word, the host; none or more is refused.

use super::parse_restore_backup_args;

fn parse(words: &[&str]) -> Result<String, String> {
    parse_restore_backup_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn one_word_is_the_host() {
    assert_eq!(parse(&["build-mac-02"]), Ok("build-mac-02".to_string()));
}

#[test]
fn no_word_is_refused() {
    assert_eq!(parse(&[]), Err("restore-backup needs the host whose backup to restore".to_string()));
}

#[test]
fn a_second_word_is_refused_by_name() {
    assert_eq!(parse(&["build-mac-02", "--all"]), Err("unexpected word \"--all\" after the host".to_string()));
    assert_eq!(parse(&["h1", "x", "y"]), Err("unexpected word \"x\" after the host".to_string()));
}
