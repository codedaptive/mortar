//! Ordered would-merge trials through the CLI, using real squashes in throwaway repositories.
use super::fixture::Landing;
use crate::{json::{check, Value}, support::raw};
use land::SquashAnswer;

#[test]
fn ordered_squashes_merge_cleanly_without_moving_refs() {
    let s = Landing::without_remote(false);
    let peer = s.peer_squash();
    let before = s.observe();
    let r = check(s.command("would-merge", Some(s.squash.commit.as_str()))
        .args(["--with", peer.commit.as_str()]).output().unwrap(), "would-merge", 0, "merges");
    assert_eq!(r.text("change_id"), s.squash.change_id.as_str());
    assert_eq!(r.text("commit"), s.squash.commit.as_str());
    assert!(r.is_null("owner"));
    assert_eq!(r.strings("lines"), &[
        format!("change {} (commit {}) merges cleanly into integration", s.squash.change_id, s.squash.commit),
        format!("then change {} (commit {}) merges cleanly onto step 1", peer.change_id, peer.commit),
        "no ref moved; any merge commit the trial wrote is reached by nothing".to_string(),
    ]);
    assert_eq!(r.text("next"), format!("mr land {} --actor <name>, then mr land {} --actor <name>", s.squash.commit, peer.commit));
    assert_eq!(s.observe(), before);
}

#[test]
fn second_squash_conflicts_with_first_result_without_moving_refs() {
    let s = Landing::without_remote(false);
    let zone = s.f.commit(&s.f.base, &[("src/file", b"peer work\n")], "peer work\n");
    raw(&s.f.git, &["update-ref", "refs/heads/zone/peer", zone.as_str()]);
    let SquashAnswer::Made(peer) = s.locked(|| land::squash(&s.f.git, &s.refs, "peer", "land-flow-peer").unwrap()) else { panic!("expected peer squash") };
    s.run("would-merge", Some(peer.commit.as_str()), 0, "merges");
    let before = s.observe();
    let r = check(s.command("would-merge", Some(s.squash.commit.as_str()))
        .args(["--with", peer.commit.as_str()]).output().unwrap(), "would-merge", 2, "conflict");
    assert_eq!(r.text("change_id"), s.squash.change_id.as_str());
    assert_eq!(r.text("commit"), s.squash.commit.as_str());
    assert_eq!(r.text("owner"), "the change's zone");
    assert_eq!(r.strings("files"), &["src/file"]);
    assert_eq!(r.strings("lines"), &[
        format!("step 2: change {} (commit {}) conflicts with integration after step 1 in:", peer.change_id, peer.commit),
        "src/file".to_string(),
        "no ref moved; any merge commit the trial wrote is reached by nothing".to_string(),
    ]);
    let step = r.object("step");
    assert_eq!(step.len(), 2);
    assert!(step.contains(&("number".into(), Value::Number(2))));
    assert!(step.contains(&("commit".into(), Value::Text(peer.commit.to_string()))));
    assert_eq!(s.observe(), before);
}

#[test]
fn with_change_id_answers_like_its_commit_without_moving_refs() {
    let s = Landing::without_remote(false);
    let peer = s.peer_squash();
    let before = s.observe();
    let by_commit = check(s.command("would-merge", Some(s.squash.commit.as_str()))
        .args(["--with", peer.commit.as_str()]).output().unwrap(), "would-merge", 0, "merges");
    let by_id = check(s.command("would-merge", Some(s.squash.commit.as_str()))
        .args(["--with", peer.change_id.as_str()]).output().unwrap(), "would-merge", 0, "merges");
    assert_eq!(by_id.text("change_id"), by_commit.text("change_id"));
    assert_eq!(by_id.text("commit"), by_commit.text("commit"));
    assert_eq!(by_id.strings("lines"), by_commit.strings("lines"));
    assert_eq!(by_id.text("next"), by_commit.text("next"));
    assert!(by_id.is_null("owner"));
    assert_eq!(s.observe(), before);
}

#[test]
fn missing_with_change_id_is_refused_like_plain_form_without_moving_refs() {
    let s = Landing::without_remote(false);
    let missing = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
    let before = s.observe();
    let plain = s.run("would-merge", Some(missing), 1, "refused");
    assert!(plain.lines().contains("no squash at refs/mortar/squash/* carries change ID"));
    let with = check(s.command("would-merge", Some(s.squash.commit.as_str()))
        .args(["--with", missing]).output().unwrap(), "would-merge", 1, "refused");
    assert_eq!(with.strings("lines"), plain.strings("lines"));
    assert_eq!(s.observe(), before);
}
