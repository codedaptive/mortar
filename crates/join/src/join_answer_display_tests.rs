//! Display for JoinAnswer: each variant's text names its ids, refs, files and next step.

use crate::JoinAnswer;
use gitio::Oid;
use std::path::PathBuf;

const FROM: &str = "1111111111111111111111111111111111111111";
const TO: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn joined_names_the_tip_and_zone_and_says_nothing_moved() {
    let text = JoinAnswer::Joined { zone_ref: "refs/heads/zone/join".to_string(), request_tip: oid(TO) }.to_string();
    assert!(text.contains("2222222222222222222222222222222222222222"), "{text}");
    assert!(text.contains("refs/heads/zone/join"), "{text}");
    assert!(text.contains("already in"), "{text}");
    assert!(text.contains("nothing moved"), "{text}");
}

#[test]
fn fast_forwarded_names_the_zone_ref_and_both_ids() {
    let answer =
        JoinAnswer::FastForwarded { zone_ref: "refs/heads/zone/join".to_string(), from: oid(FROM), to: oid(TO) };
    let text = answer.to_string();
    assert!(
        text.contains(
            "refs/heads/zone/join moved forward from 1111111111111111111111111111111111111111 to \
             2222222222222222222222222222222222222222"
        ),
        "{text}"
    );
}

#[test]
fn rebased_names_the_request_ref_ids_tests_and_next_join() {
    let answer =
        JoinAnswer::Rebased { request_ref: "refs/heads/request/r1".to_string(), from: oid(FROM), to: oid(TO) };
    let text = answer.to_string();
    assert!(
        text.contains(
            "refs/heads/request/r1 moved from 1111111111111111111111111111111111111111 to \
             2222222222222222222222222222222222222222"
        ),
        "{text}"
    );
    assert!(text.contains("onto the zone"), "{text}");
    assert!(text.contains("the unit tests must run again"), "{text}");
    assert!(text.contains("mortar join"), "{text}");
}

#[test]
fn rebased_says_nothing_about_the_zone_moving() {
    let answer =
        JoinAnswer::Rebased { request_ref: "refs/heads/request/r1".to_string(), from: oid(FROM), to: oid(TO) };
    let text = answer.to_string();
    assert!(!text.contains("zone/"), "{text}");
    assert!(!text.contains("forward"), "{text}");
    assert!(!text.contains("zone moved"), "{text}");
}

#[test]
fn conflicted_names_commit_state_each_file_on_its_own_line_and_resolve() {
    let answer = JoinAnswer::Conflicted {
        commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        state: PathBuf::from("/repo/.git/mortar-join.json"),
    };
    let text = answer.to_string();
    assert!(text.contains("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), "{text}");
    assert!(text.contains("/repo/.git/mortar-join.json"), "{text}");
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.iter().any(|line| line.trim() == "src/a.rs"), "{text}");
    assert!(lines.iter().any(|line| line.trim() == "b.txt"), "{text}");
    assert!(text.contains("mortar resolve"), "{text}");
}
