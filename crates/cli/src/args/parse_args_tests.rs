//! parse_args: globals anywhere before `--`, the verb as the verb table's entry, the rest in order, and each usage
//! error.
//! ZUT (rule 11): args/find_verb and the verb table are pure and used directly.

use super::{parse_args, ParsedArgs, UsageError, VERBS};

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

fn ok(words: &[&str]) -> ParsedArgs {
    match parse_args(&argv(words)) {
        Ok(parsed) => parsed,
        Err(error) => panic!("{words:?} gave {error:?}"),
    }
}

fn err(words: &[&str]) -> UsageError {
    match parse_args(&argv(words)) {
        Err(error) => error,
        Ok(parsed) => panic!("{words:?} gave {parsed:?}"),
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    argv(words)
}

#[test]
fn globals_after_the_verb_are_taken_out_of_rest() {
    let parsed = ok(&["init", "--origin", "main", "--json", "--actor", "h", "--remote", "o", "--key", "k"]);
    assert!(parsed.json);
    assert_eq!(parsed.key.as_deref(), Some("k"));
    assert_eq!(parsed.actor.as_deref(), Some("h"));
    assert_eq!(parsed.rest, strings(&["--origin", "main", "--remote", "o"]));
}

#[test]
fn no_globals_leaves_them_unset() {
    let parsed = ok(&["init"]);
    assert!(!parsed.json);
    assert_eq!(parsed.key, None);
    assert_eq!(parsed.actor, None);
    assert!(parsed.rest.is_empty());
}

#[test]
fn key_and_actor_with_equals() {
    let parsed = ok(&["init", "--key=a=b", "--actor=zone/req"]);
    assert_eq!(parsed.key.as_deref(), Some("a=b"));
    assert_eq!(parsed.actor.as_deref(), Some("zone/req"));
    assert!(parsed.rest.is_empty());
}

#[test]
fn words_after_double_dash_go_to_rest_as_they_are() {
    let parsed = ok(&["init", "--zone", "a=x", "--", "--json", "--key", "k", "--"]);
    assert!(!parsed.json);
    assert_eq!(parsed.key, None);
    assert_eq!(parsed.rest, strings(&["--zone", "a=x", "--json", "--key", "k", "--"]));
}

#[test]
fn a_missing_value_is_a_usage_error_naming_the_flag() {
    assert_eq!(
        err(&["init", "--key"]),
        UsageError { verb: Some("init".to_string()), reason: "--key needs a value".to_string(), json: false }
    );
    assert_eq!(err(&["init", "--actor"]).reason, "--actor needs a value");
}

#[test]
fn an_empty_value_is_a_usage_error() {
    assert_eq!(err(&["init", "--actor="]).reason, "--actor needs a non-empty value");
    assert_eq!(err(&["init", "--key", ""]).reason, "--key needs a non-empty value");
}

#[test]
fn a_repeated_global_is_a_usage_error() {
    assert_eq!(err(&["--json", "init", "--json"]).reason, "--json is given twice");
    assert_eq!(err(&["--key", "a", "init", "--key=b"]).reason, "--key is given twice");
    assert_eq!(err(&["--actor=a", "init", "--actor", "a"]).reason, "--actor is given twice");
}

#[test]
fn no_verb_is_a_usage_error_with_verb_none() {
    assert_eq!(err(&[]), UsageError { verb: None, reason: "no verb given".to_string(), json: false });
    assert_eq!(err(&["--json", "--", "init"]).verb, None);
}

#[test]
fn an_unknown_verb_is_a_usage_error_naming_it() {
    assert_eq!(
        err(&["--key", "k", "jump", "init"]),
        UsageError { verb: Some("jump".to_string()), reason: "unknown verb \"jump\"".to_string(), json: false }
    );
    assert_eq!(err(&["--origin", "init"]).verb.as_deref(), Some("--origin"));
}

#[test]
fn json_is_kept_on_a_usage_error() {
    assert!(err(&["jump", "--json"]).json);
    assert!(err(&["--json"]).json);
    let late = err(&["init", "--key", "", "--json"]);
    assert_eq!(late.reason, "--key needs a non-empty value");
    assert!(late.json);
    assert!(!err(&["jump", "--", "--json"]).json);
}

#[test]
fn dash_dash_marks_where_the_words_after_the_first_double_dash_start() {
    let a = "1111111111111111111111111111111111111111";
    let b = "2222222222222222222222222222222222222222";
    let parsed = ok(&["bisect", a, b, "--timeout", "5", "--", "make", "--actor", "x", "--json"]);
    assert_eq!(parsed.rest, strings(&[a, b, "--timeout", "5", "make", "--actor", "x", "--json"]));
    assert_eq!(parsed.dash_dash, Some(4));
    assert_eq!(parsed.actor, None);
    assert!(!parsed.json);
}

#[test]
fn a_line_without_double_dash_has_no_dash_dash() {
    assert_eq!(ok(&["init", "--origin", "main"]).dash_dash, None);
    assert_eq!(ok(&["init"]).dash_dash, None);
}

#[test]
fn a_double_dash_with_nothing_before_or_after_it_is_at_zero() {
    let parsed = ok(&["init", "--"]);
    assert!(parsed.rest.is_empty());
    assert_eq!(parsed.dash_dash, Some(0));
}

#[test]
fn only_the_first_double_dash_marks_a_later_one_is_a_plain_word() {
    let parsed = ok(&["init", "--zone", "a=x", "--", "--json", "--key", "k", "--"]);
    assert_eq!(parsed.rest, strings(&["--zone", "a=x", "--json", "--key", "k", "--"]));
    assert_eq!(parsed.dash_dash, Some(2));
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
