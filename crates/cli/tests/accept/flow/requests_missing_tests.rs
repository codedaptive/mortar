use super::requests_missing;

const SQUASH: &str = "a: 2 requests\n\n- r1: r1: first; r1: work saved at join\n- r2: r2: second\n\n\
                      Mortar-Change-Id: kzlymxnwovpuqtrssrtqupvowxnymlzk\nMortar-Zone: a\n";

#[test]
fn a_message_naming_both_misses_none() {
    assert!(requests_missing(SQUASH, &["r1", "r2"]).is_empty());
}

#[test]
fn a_request_without_its_line_is_missing() {
    assert_eq!(requests_missing(SQUASH, &["r1", "r4", "r2", "r3"]), vec!["r4".to_string(), "r3".to_string()]);
}

#[test]
fn a_name_in_a_subject_or_a_longer_name_does_not_count() {
    let message = "a: 1 requests\n\n- r10: fixes r2\n";
    assert_eq!(requests_missing(message, &["r1", "r2"]), vec!["r1".to_string(), "r2".to_string()]);
}
