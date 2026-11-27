//! CLI bring_zone_folder/report joined to Git's actual staged, unstaged, untracked and unmerged indexes.
use super::fixture::World;
use crate::support::{raw, write};
use gitio::RunOpts;
use std::fs;

#[test]
fn staged_work_at_a_path_the_published_tip_changes_is_kept_then_clear_and_repeat_finishes() {
    let w = World::new(); w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish();
    write(&w.s.f.zone, "src/file", b"owner's staged bytes\n");
    raw(&w.git(&w.s.f.zone), &["add", "src/file"]); let folder = w.folder_state(&w.s.f.zone);
    let r = w.after(3, "folder-not-moved");
    assert_eq!(r.strings("moved"), &["core", "b"]);
    assert_eq!(r.strings("folders_refused"), &[w.s.f.zone.display().to_string()]);
    assert_eq!(r.strings("folders_moved"), &[w.b.display().to_string()]);
    assert_eq!(r.text("owner"), "the zone folders listed");
    assert!(r.strings("lines").contains(&format!("the zone folder {} was not brought to {origin}: its index matches no commit zone core has held", w.s.f.zone.display())));
    assert_eq!(r.text("next"), "take the staged changes out of each zone folder whose index matches no zone commit, without resetting its index to HEAD alone; then repeat the same command: mr after-publish core --move b --actor <name>");
    assert_eq!(w.folder_state(&w.s.f.zone), folder);
    assert_eq!(raw(&w.git(&w.s.f.zone), &["show", ":src/file"]), b"owner's staged bytes\n");
    w.folder_at(&w.b, &origin, &[("src/file", b"published\n")]); w.assert_origin(&origin);
    // Keep both staged and on-disk content outside the folder, then put its actual old index and files back.
    let held = w.s.f.root.join("staged-work-kept"); fs::rename(w.s.f.zone.join("src/file"), &held).unwrap();
    raw(&w.git(&w.s.f.zone), &["read-tree", w.s.f.base.as_str()]); write(&w.s.f.zone, "src/file", b"base\n");
    let before = w.store(); let again = w.after(0, "after-published");
    assert_eq!(again.strings("current"), &["core", "b"]); assert!(again.strings("moved").is_empty());
    assert_eq!(again.strings("folders_moved"), &[w.s.f.zone.display().to_string()]); assert_eq!(w.store(), before);
    assert_eq!(fs::read(held).unwrap(), b"owner's staged bytes\n");
    w.folder_at(&w.s.f.zone, &origin, &[("src/file", b"published\n")]);
}

#[test]
fn unstaged_untracked_and_unmerged_refusals_preserve_every_byte_leave_other_zones_done_and_repeat() {
    for kind in ["unstaged", "untracked", "unmerged"] {
        let w = World::new(); w.core(&[("src/file", b"published\n"), ("src/new", b"new upstream\n")]);
        let (_, origin) = w.publish(); let folder = &w.s.f.zone; let git = w.git(folder);
        let path = if kind == "untracked" { "src/new" } else { "src/file" };
        write(folder, path, b"owner's obstruction\n");
        if kind == "unmerged" {
            let base = raw(&git, &["rev-parse", &format!("{}:src/file", w.s.f.base)]);
            let incoming = raw(&git, &["rev-parse", &format!("{origin}:src/file")]);
            let oid = |bytes: Vec<u8>| String::from_utf8(bytes).unwrap().trim().to_string();
            let base = oid(base); let incoming = oid(incoming);
            let data = format!("0 0000000000000000000000000000000000000000\tsrc/file\n100644 {base} 1\tsrc/file\n100644 {base} 2\tsrc/file\n100644 {incoming} 3\tsrc/file\n");
            git.run(&["update-index", "--index-info"], &RunOpts { stdin: Some(data.into_bytes()), ..RunOpts::default() }).unwrap();
            assert!(!raw(&git, &["ls-files", "-u"]).is_empty());
        }
        let before_folder = w.folder_state(folder); let r = w.after(3, "folder-not-moved");
        assert_eq!(r.strings("folders_refused"), &[folder.display().to_string()]);
        assert_eq!(r.strings("folders_moved"), &[w.b.display().to_string()]);
        let reason = if kind == "unmerged" { "its index has unmerged entries at:" } else { "local changes are in the way at:" };
        assert!(r.strings("lines").contains(&format!("the zone folder {} was not brought to {origin}: {reason}", folder.display())));
        assert!(r.strings("lines").contains(&path.to_string())); assert!(r.strings("lines").contains(&"nothing in it changed".into()));
        let next = if kind == "unmerged" { "finish or undo the merge in each zone folder whose index has unmerged entries, leaving no unmerged entry" }
            else { "move each path listed as in the way out of its zone folder, or put it back to that folder's index copy, and leave the index as it is" };
        assert_eq!(r.text("next"), format!("{next}; then repeat the same command: mr after-publish core --move b --actor <name>"));
        assert_eq!(w.folder_state(folder), before_folder); w.folder_at(&w.b, &origin, &[("src/file", b"published\n")]);
        let held = w.s.f.root.join(format!("kept-{kind}")); fs::rename(folder.join(path), &held).unwrap();
        if kind == "unmerged" { raw(&git, &["read-tree", w.s.f.base.as_str()]); }
        if path == "src/file" { write(folder, path, b"base\n"); }
        let before = w.store(); let repeat = w.after(0, "after-published");
        assert_eq!(repeat.strings("current"), &["core", "b"]); assert_eq!(repeat.strings("folders_moved"), &[folder.display().to_string()]);
        assert_eq!(w.store(), before); assert_eq!(fs::read(held).unwrap(), b"owner's obstruction\n");
        w.folder_at(folder, &origin, &[("src/file", b"published\n"), ("src/new", b"new upstream\n")]); w.assert_origin(&origin);
    }
}

#[test]
fn squash_of_not_in_publishing_zone_refuses_with_the_real_zone_and_source_before_folder_moves() {
    let w = World::new(); let source = w.core(&[("src/file", b"squashed source\n")]);
    let (_, origin) = w.publish();
    // A real related branch that lacks the squash's source and any recorded replay of it.
    let other = w.s.f.commit(&w.s.f.base, &[("src/other", b"not that source\n")], "other lineage\n");
    raw(&w.s.f.git, &["update-ref", "refs/heads/zone/core", other.as_str()]);
    let before = w.store(); let folders = (w.folder_state(&w.s.f.zone), w.folder_state(&w.b));
    let r = w.after(3, "refused");
    assert_eq!(r.strings("lines")[0], format!("zone core's published squash was made from {source}, which zone core does not hold, itself or as a recorded replay"));
    assert_eq!(r.strings("lines")[1], "any zone named before it was already moved; no zone folder was brought to its tip");
    assert_eq!(r.text("owner"), "zone core"); assert!(r.text("next").contains(&format!("{source}")));
    assert_eq!(w.store(), before); assert_eq!((w.folder_state(&w.s.f.zone), w.folder_state(&w.b)), folders); w.assert_origin(&origin);
}
