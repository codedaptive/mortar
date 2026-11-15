//! log_filters: --ref read once from the words, then the --actor and --key flags, in that order; anything else is a
//! reason for log's usage. ZUT: oplog's HistoryFilter is used as plain data.

use oplog::HistoryFilter;

use super::log_filters;

fn words(words: &[&str]) -> Vec<String> {
    words.iter().map(ToString::to_string).collect()
}

#[test]
fn no_words_and_no_flags_is_no_filter() {
    assert_eq!(log_filters(&[], None, None), Ok(Vec::new()));
}

#[test]
fn ref_in_either_form_then_actor_then_key() {
    let expected = vec![
        HistoryFilter::Ref("refs/heads/zone/core".to_string()),
        HistoryFilter::Actor("bob".to_string()),
        HistoryFilter::Key("k-1".to_string()),
    ];
    assert_eq!(log_filters(&words(&["--ref", "refs/heads/zone/core"]), Some("k-1"), Some("bob")), Ok(expected.clone()));
    assert_eq!(log_filters(&words(&["--ref=refs/heads/zone/core"]), Some("k-1"), Some("bob")), Ok(expected));
}

#[test]
fn actor_or_key_alone_is_that_one_filter() {
    assert_eq!(log_filters(&[], None, Some("bob")), Ok(vec![HistoryFilter::Actor("bob".to_string())]));
    assert_eq!(log_filters(&[], Some("k-1"), None), Ok(vec![HistoryFilter::Key("k-1".to_string())]));
}

#[test]
fn each_bad_line_is_its_reason() {
    let cases: [(&[&str], &str); 5] = [
        (&["oops"], "unknown word \"oops\""),
        (&["--ref"], "--ref needs a value"),
        (&["--ref="], "--ref needs a non-empty value"),
        (&["--ref", ""], "--ref needs a non-empty value"),
        (&["--ref", "refs/heads/a", "--ref=refs/heads/b"], "--ref is given twice"),
    ];
    for (line, reason) in cases {
        assert_eq!(log_filters(&words(line), None, None), Err(reason.to_string()), "{line:?}");
    }
}

#[test]
fn a_word_after_a_good_ref_is_still_refused() {
    let line = words(&["--ref", "refs/heads/a", "extra"]);
    assert_eq!(log_filters(&line, None, None), Err("unknown word \"extra\"".to_string()));
}
