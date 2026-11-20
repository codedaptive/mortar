//! parse_would_merge_args: one word, read as a full commit id or else a change ID, and each word count or word it
//! refuses.

use gitio::Oid;
use oplog::ChangeId;

use super::parse_would_merge_args;
use crate::landing::SquashName;

const SHA1: &str = "abcdef0123456789abcdef0123456789abcdef01";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn parse(words: &[&str]) -> Result<SquashName, String> {
    parse_would_merge_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn a_full_commit_id_is_a_commit() {
    assert_eq!(parse(&[SHA1]), Ok(SquashName::Commit(Oid::parse(SHA1).unwrap())));
    assert_eq!(parse(&[SHA256]), Ok(SquashName::Commit(Oid::parse(SHA256).unwrap())));
}

#[test]
fn a_change_id_is_a_change() {
    assert_eq!(parse(&[CHANGE]), Ok(SquashName::Change(ChangeId::parse(CHANGE).unwrap())));
}

#[test]
fn no_word_is_refused() {
    assert_eq!(parse(&[]), Err("would-merge needs a squash: its commit id or its change ID".to_string()));
}

#[test]
fn a_second_word_is_refused_by_name() {
    assert_eq!(parse(&[SHA1, "--now"]), Err("unexpected word \"--now\" after the squash".to_string()));
    assert_eq!(parse(&[CHANGE, "x", "y"]), Err("unexpected word \"x\" after the squash".to_string()));
}

#[test]
fn a_word_that_is_neither_is_refused_by_name() {
    const NEITHER: &str = " is neither a full commit id (40 or 64 lowercase hex characters) nor a change ID (32 letters \
                           k..z)";
    let cases = [
        ("HEAD", "\"HEAD\""),
        ("abc1234", "\"abc1234\""),
        ("ABCDEF0123456789ABCDEF0123456789ABCDEF01", "\"ABCDEF0123456789ABCDEF0123456789ABCDEF01\""),
        ("kkkkllllmmmmnnnnooooppppqqqqrrr", "\"kkkkllllmmmmnnnnooooppppqqqqrrr\""),
        ("aaaabbbbccccddddeeeeffffgggghhhh", "\"aaaabbbbccccddddeeeeffffgggghhhh\""),
    ];
    for (word, quoted) in cases {
        assert_eq!(parse(&[word]), Err(format!("{quoted}{NEITHER}")), "word {word}");
    }
}
