//! parse_rereview_args: two full commit ids, and each word count or word it refuses.

use gitio::Oid;

use super::parse_rereview_args;

const SHA1_A: &str = "1111111111111111111111111111111111111111";
const SHA1_B: &str = "abcdef0123456789abcdef0123456789abcdef01";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn parse(words: &[&str]) -> Result<(Oid, Oid), String> {
    parse_rereview_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

/// The two ids the parse gave, as text, in order.
fn ids(words: &[&str]) -> Result<(String, String), String> {
    parse(words).map(|(reviewed, new)| (reviewed.as_str().to_string(), new.as_str().to_string()))
}

#[test]
fn two_full_commit_ids_are_the_reviewed_and_the_new_in_order() {
    assert_eq!(ids(&[SHA1_A, SHA1_B]), Ok((SHA1_A.to_string(), SHA1_B.to_string())));
    assert_eq!(ids(&[SHA1_B, SHA1_A]), Ok((SHA1_B.to_string(), SHA1_A.to_string())));
    assert_eq!(ids(&[SHA256, SHA1_A]), Ok((SHA256.to_string(), SHA1_A.to_string())));
}

#[test]
fn fewer_than_two_words_is_refused() {
    let reason = Err("rereview needs two commits: the reviewed squash and the new one".to_string());
    assert_eq!(ids(&[]), reason);
    assert_eq!(ids(&[SHA1_A]), reason);
}

#[test]
fn a_word_after_the_two_commits_is_refused_by_name() {
    assert_eq!(ids(&[SHA1_A, SHA1_B, "--now"]), Err("unexpected word \"--now\" after the two commits".to_string()));
    assert_eq!(ids(&[SHA1_A, SHA1_B, "x", "y"]), Err("unexpected word \"x\" after the two commits".to_string()));
}

#[test]
fn a_word_that_is_not_a_full_commit_id_is_refused_by_name() {
    assert_eq!(
        ids(&["HEAD", SHA1_B]),
        Err("\"HEAD\" is not a full commit id (40 or 64 lowercase hex characters)".to_string())
    );
    assert_eq!(
        ids(&[SHA1_A, "abc1234"]),
        Err("\"abc1234\" is not a full commit id (40 or 64 lowercase hex characters)".to_string())
    );
    assert_eq!(
        ids(&[SHA1_A, "ABCDEF0123456789ABCDEF0123456789ABCDEF01"]),
        Err("\"ABCDEF0123456789ABCDEF0123456789ABCDEF01\" is not a full commit id (40 or 64 lowercase hex characters)"
            .to_string())
    );
}
