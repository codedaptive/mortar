use super::trailers_missing;

const ALL: &str = "Mortar-Change-Id: kzlymxnwovpuqtrssrtqupvowxnymlzk\nMortar-Request: r1\nMortar-Zone: a\n";

#[test]
fn all_three_for_the_request_and_zone_lack_nothing() {
    assert!(trailers_missing(ALL, "r1", "a").is_empty());
}

#[test]
fn no_trailers_lack_all_three() {
    assert_eq!(
        trailers_missing("", "r1", "a"),
        vec!["Mortar-Change-Id".to_string(), "Mortar-Request: r1".to_string(), "Mortar-Zone: a".to_string()]
    );
}

#[test]
fn another_request_or_zone_is_named() {
    assert_eq!(trailers_missing(ALL, "r2", "b"), vec!["Mortar-Request: r2".to_string(), "Mortar-Zone: b".to_string()]);
}

#[test]
fn an_empty_change_id_is_missing() {
    let text = "Mortar-Change-Id: \nMortar-Request: r1\nMortar-Zone: a\n";
    assert_eq!(trailers_missing(text, "r1", "a"), vec!["Mortar-Change-Id".to_string()]);
}

#[test]
fn keys_match_whatever_their_case_and_the_last_one_counts() {
    let text = "mortar-change-id: kzlymxnwovpuqtrssrtqupvowxnymlzk\nMORTAR-REQUEST: r9\nMortar-Request: r1\n\
                mortar-zone: a\n";
    assert!(trailers_missing(text, "r1", "a").is_empty());
}

#[test]
fn other_trailers_are_ignored() {
    let text = "Signed-off-by: someone\nMortar-Squash-Of: 0a1b\n";
    assert_eq!(trailers_missing(text, "r1", "a").len(), 3);
}
