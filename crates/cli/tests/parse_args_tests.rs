//! parse_args: globals before the verb are read, and the verb is the verb table's entry.
//! ZIT (S-79 as replaced: the line is the file): it calls args/find_verb for real, which reads args/verbs (the verb table).

use cli::{parse_args, ParsedArgs, VERBS};

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

fn ok(words: &[&str]) -> ParsedArgs {
    match parse_args(&argv(words)) {
        Ok(parsed) => parsed,
        Err(error) => panic!("{words:?} gave {error:?}"),
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    argv(words)
}

#[test]
fn globals_before_the_verb() {
    let parsed = ok(&["--json", "--key", "k1", "--actor", "harness", "init", "--origin", "main"]);
    assert_eq!(parsed.verb.name, "init");
    assert_eq!(parsed.verb, VERBS.iter().find(|verb| verb.name == "init").expect("init is in the table"));
    assert!(parsed.json);
    assert_eq!(parsed.key.as_deref(), Some("k1"));
    assert_eq!(parsed.actor.as_deref(), Some("harness"));
    assert_eq!(parsed.rest, strings(&["--origin", "main"]));
}
