//! Undo restores the prior request boundary for a rebase and a completed resolve.
use super::fixture::{ours, rebase, World};

#[test]
fn base_undo_plain_join_and_rejoin() {
    let w = World::new(); let (_, _, zone) = rebase(&w); let f = &w.s.f;
    ours(&w, &["undo"], 0, "undone");
    assert_eq!(f.status().text("base"), f.base.as_str());
    ours(&w, &["join"], 0, "rebased");
    assert_eq!(f.status().text("base"), zone.as_str());
}

#[test]
fn base_undo_resolve_and_resolve_again() {
    let w = World::new(); let f = &w.s.f; f.save_topic(); let zone = f.advance_zone(true);
    ours(&w, &["join"], 2, "conflicted"); ours(&w, &["resolve"], 2, "files-written");
    f.write("src/file", b"chosen resolution\n");
    ours(&w, &["resolve"], 0, "rebased");
    assert_eq!(f.status().text("base"), zone.as_str());
    ours(&w, &["undo"], 0, "undone");
    assert_eq!(f.status().text("base"), f.base.as_str());
    let status = ours(&w, &["status"], 0, "paused"); assert_eq!(status.text("next"), "mr resolve");
    ours(&w, &["resolve"], 0, "rebased");
    assert_eq!(f.status().text("base"), zone.as_str());
}
