//! parse_would_merge_args: the squash word read as a full commit id or else a change ID, each `--with` word read
//! the same way in order wherever it comes, and each word count or word it refuses.

use gitio::Oid;
use oplog::ChangeId;

use super::{parse_would_merge_args, WouldMergeWords};
use crate::landing::SquashName;

const SHA1: &str = "abcdef0123456789abcdef0123456789abcdef01";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const OTHER_CHANGE: &str = "zzzzyyyyxxxxwwwwvvvvuuuuttttssss";
const NO_WORD: &str = "would-merge needs a squash: its commit id or its change ID";
const WITH_NO_WORD: &str = "--with needs a squash: its commit id or its change ID";
const NEITHER: &str = " is neither a full commit id (40 or 64 lowercase hex characters) nor a change ID (32 letters k..z)";

fn commit(word: &str) -> SquashName {
    SquashName::Commit(Oid::parse(word).unwrap())
}

fn change(word: &str) -> SquashName {
    SquashName::Change(ChangeId::parse(word).unwrap())
}

fn words(squash: SquashName, with: Vec<SquashName>) -> Result<WouldMergeWords, String> {
    Ok(WouldMergeWords { squash, with })
}

fn neither(quoted: &str) -> Result<WouldMergeWords, String> {
    Err(format!("{quoted}{NEITHER}"))
}

#[test]
fn reads_the_squash_and_each_with_or_names_the_refusal() {
    let rows: Vec<(&str, Vec<&str>, Result<WouldMergeWords, String>)> = vec![
        ("sha1 commit", vec![SHA1], words(commit(SHA1), vec![])),
        ("sha256 commit", vec![SHA256], words(commit(SHA256), vec![])),
        ("change id", vec![CHANGE], words(change(CHANGE), vec![])),
        ("no word", vec![], Err(NO_WORD.to_string())),
        ("second word", vec![SHA1, "--now"], Err("unexpected word \"--now\" after the squash".to_string())),
        ("first of extras", vec![CHANGE, "x", "y"], Err("unexpected word \"x\" after the squash".to_string())),
        ("extra before neither", vec!["HEAD", "x"], Err("unexpected word \"x\" after the squash".to_string())),
        ("neither HEAD", vec!["HEAD"], neither("\"HEAD\"")),
        ("neither short", vec!["abc1234"], neither("\"abc1234\"")),
        (
            "neither upper hex",
            vec!["ABCDEF0123456789ABCDEF0123456789ABCDEF01"],
            neither("\"ABCDEF0123456789ABCDEF0123456789ABCDEF01\""),
        ),
        ("neither 31 letters", vec!["kkkkllllmmmmnnnnooooppppqqqqrrr"], neither("\"kkkkllllmmmmnnnnooooppppqqqqrrr\"")),
        ("neither a..h", vec!["aaaabbbbccccddddeeeeffffgggghhhh"], neither("\"aaaabbbbccccddddeeeeffffgggghhhh\"")),
        ("one with", vec![SHA1, "--with", CHANGE], words(commit(SHA1), vec![change(CHANGE)])),
        (
            "withs in order, squash between",
            vec!["--with", OTHER_CHANGE, CHANGE, "--with", SHA256, "--with", SHA1],
            words(change(CHANGE), vec![change(OTHER_CHANGE), commit(SHA256), commit(SHA1)]),
        ),
        ("with alone", vec!["--with", SHA1], Err(NO_WORD.to_string())),
        ("with no word", vec![SHA1, "--with"], Err(WITH_NO_WORD.to_string())),
        ("bare with", vec!["--with"], Err(WITH_NO_WORD.to_string())),
        ("with word neither", vec![SHA1, "--with", "HEAD"], neither("\"HEAD\"")),
        ("squash neither before with", vec!["main", "--with", "HEAD"], neither("\"main\"")),
        ("extra after with", vec![SHA1, "--with", CHANGE, "x"], Err("unexpected word \"x\" after the squash".to_string())),
    ];
    for (name, given, expected) in rows {
        let rest: Vec<String> = given.iter().map(ToString::to_string).collect();
        assert_eq!(parse_would_merge_args(&rest), expected, "row {name}");
    }
}
