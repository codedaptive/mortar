//! Joins finish_stopped_move through CLI land/back-out/publish with real Land ops, Folder moves and Git indexes.
//! S-137(3)/(4): a blocked finish stops BEFORE the verb's own call; clearing preserves the index, not HEAD.
use super::fixture::Landing;
use crate::{json::Report, support::{raw, write}};
use gitio::{Oid, RunOpts};

#[test]
fn refused_pre_act_finish_stops_each_verb_before_its_own_move_or_push_then_clear_and_repeat_finishes() {
    for verb in ["land", "back-out", "publish"] {
        let s = Landing::new(true);
        let (old, tip) = if verb == "land" {
            s.land(); s.kit_back_out(); (s.squash.commit.clone(), s.f.base.clone())
        } else { s.kit_land(); (s.f.base.clone(), s.squash.commit.clone()) };
        s.obstruct(); let before = s.observe();
        let r = s.run(verb, word(&s, verb), 1, "folder-not-moved");
        assert_eq!(s.observe(), before, "{verb} must not move integration, publish, consume state, log or touch the folder");
        assert_eq!(r.strings("lines")[0], format!("{verb} stopped before its land call and moved nothing: integration is at {tip}, where an earlier run moved it, and its folder is still at {old}"));
        refusal(&s, &r, verb, &old, &tip);
        let index = s.index(); s.clear_to_index(); assert_eq!(s.index(), index);
        let r = s.run(verb, word(&s, verb), 0, if verb == "land" { "landed" } else if verb == "back-out" { "backed-out" } else { "published" });
        s.assert_finished(&r, &old, &tip);
        let final_tip = if verb == "back-out" { &s.f.base } else { &s.squash.commit };
        s.assert_clean_at(final_tip, if verb == "back-out" { b"base\n" } else { b"zone work\n" });
        if verb == "publish" {
            assert_eq!(raw(&gitio::Git { repo: s.bare.clone(), ..s.f.git.clone() }, &["rev-parse", "refs/heads/main"]), format!("{}\n", s.squash.commit).as_bytes());
        }
    }
}

#[test]
fn refused_own_folder_move_can_be_finished_on_the_plain_repeat_of_each_verb() {
    for verb in ["land", "back-out", "publish"] {
        let s = Landing::new(true);
        let (old, tip) = if verb == "land" { (s.f.base.clone(), s.squash.commit.clone()) }
        else {
            s.land();
            let tip = if verb == "publish" { s.remote_move() } else { s.f.base.clone() };
            (s.squash.commit.clone(), tip)
        };
        s.obstruct(); let index = s.index(); let bytes = s.bytes();
        let r = s.run(verb, word(&s, verb), 1, "folder-not-moved");
        assert_eq!(s.tip(), tip); assert_eq!(s.index(), index); assert_eq!(s.bytes(), bytes);
        assert!(!r.lines().contains("stopped before its land call")); refusal(&s, &r, verb, &old, &tip);
        s.clear_to_index(); assert_eq!(s.index(), index);
        let r = s.run(verb, word(&s, verb), 0, repeat_outcome(verb));
        s.assert_finished(&r, &old, &tip);
        s.assert_clean_at(&tip, if verb == "land" { b"zone work\n" } else if verb == "back-out" { b"base\n" } else { b"remote work\n" });
        let before = s.observe(); s.run(verb, word(&s, verb), 0, repeat_outcome(verb)); assert_eq!(s.observe(), before);
    }
}

#[test]
fn stopped_back_out_and_origin_moved_publish_finish_even_when_the_kit_repeat_answers_nothing() {
    for verb in ["back-out", "publish"] {
        let s = Landing::new(true); s.land();
        let old = s.squash.commit.clone();
        let tip = if verb == "back-out" { s.kit_back_out(); s.f.base.clone() }
        else {
            let tip = s.remote_move();
            assert_eq!(s.locked(|| land::publish(&s.f.git, &s.refs, "rit6-setup").unwrap()), land::PublishAnswer::OriginMoved { found: tip.clone() });
            tip
        };
        assert_eq!(s.bytes(), b"zone work\n");
        let records = s.f.records();
        let r = s.run(verb, None, 0, repeat_outcome(verb)); s.assert_finished(&r, &old, &tip);
        s.assert_clean_at(&tip, if verb == "back-out" { b"base\n" } else { b"remote work\n" });
        assert_eq!(s.f.records(), records, "catch-up logs no second integration move");
    }
}

#[test]
fn staged_and_unmerged_indexes_are_not_stalls_and_are_left_byte_for_byte_while_the_verb_goes_on() {
    for unmerged in [false, true] {
        let s = Landing::new(true); s.land(); s.kit_back_out();
        if unmerged {
            let other = s.f.commit(&s.f.base, &[("src/file", b"other conflict\n")], "conflicting index\n");
            raw(&s.folder_git(), &["read-tree", "-m", s.f.base.as_str(), s.squash.commit.as_str(), other.as_str()]);
            assert!(!raw(&s.folder_git(), &["ls-files", "-u"]).is_empty());
        } else {
            write(&s.folder, "src/file", b"staged owner work\n"); raw(&s.folder_git(), &["add", "src/file"]);
        }
        let before = s.observe();
        let r = s.run("publish", None, 0, "nothing-to-publish");
        assert!(!r.lines().contains("first finished a stopped move")); assert_eq!(s.observe(), before);
    }
}

#[test]
fn land_with_a_staged_non_stall_index_goes_on_to_its_busy_answer_without_touching_the_folder() {
    let s = Landing::new(true); let peer = s.peer_squash(); s.kit_land();
    write(&s.folder, "src/file", b"staged owner work\n"); raw(&s.folder_git(), &["add", "src/file"]);
    let before = s.observe(); let r = s.run("land", Some(peer.commit.as_str()), 3, "busy");
    assert_eq!(r.text("busy_with"), s.squash.change_id.as_str());
    assert!(!r.lines().contains("first finished a stopped move")); assert_eq!(s.observe(), before);
}

#[test]
fn no_integration_folder_or_no_done_integration_op_means_no_catch_up() {
    let s = Landing::new(false); s.kit_land();
    let r = s.land(); assert!(r.is_null("folder")); assert!(!r.lines().contains("first finished"));
    let s = Landing::new(true);
    raw(&s.f.git, &["update-ref", "refs/heads/integration", s.squash.commit.as_str()]);
    // The index is at base and branch elsewhere, but no Done op moved integration: do not guess a stall.
    let before = s.observe(); let r = s.run("publish", None, 0, "nothing-to-publish");
    assert!(!r.lines().contains("first finished")); assert_eq!(s.observe(), before);
}

#[test]
fn setting_the_index_to_head_removes_the_stall_and_does_not_invent_a_worktree_content_heuristic() {
    let s = Landing::new(true); s.land(); s.kit_back_out(); s.obstruct();
    s.run("publish", None, 1, "folder-not-moved");
    // Deliberately NOT the prescribed clear: index at HEAD, owner's file still differs. S-137(4) leaves it unreported.
    s.folder_git().run(&["read-tree", "HEAD"], &RunOpts::default()).unwrap();
    let before = s.observe(); let r = s.run("publish", None, 0, "nothing-to-publish");
    assert!(!r.lines().contains("first finished")); assert_eq!(s.observe(), before);
    assert_eq!(s.bytes(), b"owner's unsaved work\n");
}

fn word<'a>(s: &'a Landing, verb: &str) -> Option<&'a str> { (verb == "land").then_some(s.squash.commit.as_str()) }
fn repeat_outcome(verb: &str) -> &str { match verb { "land" => "landed", "back-out" => "nothing-to-back-out", "publish" => "nothing-to-publish", _ => unreachable!() } }
fn refusal(s: &Landing, r: &Report, verb: &str, old: &Oid, tip: &Oid) {
    assert_eq!(r.text("folder"), s.folder.to_str().unwrap()); assert_eq!(r.text("old"), old.as_str()); assert_eq!(r.text("new"), tip.as_str());
    assert_eq!(r.strings("paths"), &["src/file"]); assert!(r.text("owner").contains(s.folder.to_str().unwrap()));
    assert!(r.strings("lines").iter().any(|line| line == "src/file"));
    assert_eq!(r.text("next"), format!("move each path listed out of the integration folder {}, or put it back to the folder's index copy, and leave the index as it is; then repeat the same command: mr {verb}{} --actor <name>", s.folder.display(), if verb == "land" { format!(" {}", s.squash.commit) } else { String::new() }));
    assert!(!r.text("next").contains("git "));
}
