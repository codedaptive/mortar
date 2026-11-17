//! parse_land_args: one full commit id, and each word count or word it refuses; a change ID is refused with the
//! way to find its commit.

use gitio::Oid;

use super::parse_land_args;

const SHA1: &str = "abcdef0123456789abcdef0123456789abcdef01";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn parse(words: &[&str]) -> Result<Oid, String> {
    parse_land_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn a_full_commit_id_is_the_commit() {
    assert_eq!(parse(&[SHA1]), Ok(Oid::parse(SHA1).unwrap()));
    assert_eq!(parse(&[SHA256]), Ok(Oid::parse(SHA256).unwrap()));
}

#[test]
fn no_word_is_refused() {
    assert_eq!(parse(&[]), Err("land needs the squash's commit id".to_string()));
}

#[test]
fn a_second_word_is_refused_by_name() {
    assert_eq!(parse(&[SHA1, "--now"]), Err("unexpected word \"--now\" after the commit".to_string()));
    assert_eq!(parse(&[SHA1, "x", "y"]), Err("unexpected word \"x\" after the commit".to_string()));
}

#[test]
fn a_change_id_is_refused_naming_would_merge_to_find_the_commit() {
    assert_eq!(
        parse(&[CHANGE]),
        Err("\"kkkkllllmmmmnnnnooooppppqqqqrrrr\" is a change ID; land takes the squash's commit id only (find it \
             with mortar would-merge kkkkllllmmmmnnnnooooppppqqqqrrrr --actor <name>)"
            .to_string())
    );
}

#[test]
fn a_word_that_is_not_a_full_commit_id_is_refused_by_name() {
    assert_eq!(parse(&["HEAD"]), Err("\"HEAD\" is not a full commit id (40 or 64 lowercase hex characters)".to_string()));
    assert_eq!(
        parse(&["abc1234"]),
        Err("\"abc1234\" is not a full commit id (40 or 64 lowercase hex characters)".to_string())
    );
    assert_eq!(
        parse(&["ABCDEF0123456789ABCDEF0123456789ABCDEF01"]),
        Err("\"ABCDEF0123456789ABCDEF0123456789ABCDEF01\" is not a full commit id (40 or 64 lowercase hex characters)"
            .to_string())
    );
    assert_eq!(
        parse(&["kkkkllllmmmmnnnnooooppppqqqqrrr"]),
        Err("\"kkkkllllmmmmnnnnooooppppqqqqrrr\" is not a full commit id (40 or 64 lowercase hex characters)"
            .to_string())
    );
}
