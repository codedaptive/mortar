//! Joins the `mr would-merge` binary command with Land's squash lookup/merge check and real Git/Oplog locks.
use super::fixture::Landing;
use crate::support::raw;

#[test]
fn yes_and_no_by_full_commit_and_change_id_leave_refs_log_folder_and_commit_inventory_unchanged() {
    for conflict in [false, true] {
        let s = Landing::new(true);
        if conflict {
            let next = s.f.commit(&s.f.base, &[("src/file", b"origin work\n")], "origin work\n");
            raw(&s.f.git, &["update-ref", "refs/heads/main", next.as_str()]);
            raw(&s.f.git, &["update-ref", "refs/heads/integration", next.as_str()]);
            s.sync(&s.f.base, &next);
        }
        for word in [s.squash.commit.as_str(), s.squash.change_id.as_str()] {
            let before = s.observe(); let commits = commits(&s);
            let r = s.run("would-merge", Some(word), if conflict { 2 } else { 0 }, if conflict { "conflict" } else { "merges" });
            assert_eq!(r.text("change_id"), s.squash.change_id.as_str());
            assert_eq!(r.text("commit"), s.squash.commit.as_str());
            if conflict { assert_eq!(r.strings("files"), &["src/file"]); }
            else { assert_eq!(r.text("next"), format!("mr land {} --actor <name>", s.squash.commit)); }
            assert!(r.lines().contains("no ref moved and no commit was written"));
            assert_eq!(s.observe(), before); assert_eq!(self::commits(&s), commits);
        }
    }
}

#[test]
fn missing_and_duplicate_change_ids_are_refused_without_movement_and_name_the_ambiguity() {
    let s = Landing::new(true);
    let before = s.observe();
    let r = s.run("would-merge", Some("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), 1, "refused");
    assert!(r.lines().contains("no squash at refs/mortar/squash/* carries change ID"));
    assert!(r.text("next").contains("mr squash <zone>")); assert_eq!(s.observe(), before);
    raw(&s.f.git, &["update-ref", "refs/mortar/squash/duplicate", s.squash.commit.as_str()]);
    let before = s.observe();
    let r = s.run("would-merge", Some(s.squash.change_id.as_str()), 1, "refused");
    assert!(r.lines().contains("refs/mortar/squash/core, refs/mortar/squash/duplicate"));
    assert!(r.text("next").contains("mr would-merge <commit>")); assert_eq!(s.observe(), before);
    s.run("would-merge", Some(s.squash.commit.as_str()), 0, "merges");
    assert_eq!(s.observe(), before);
}

#[test]
fn a_commit_without_mortar_trailers_is_refused_but_a_bad_ref_beside_a_good_ref_is_skipped() {
    let s = Landing::new(true);
    raw(&s.f.git, &["update-ref", "refs/mortar/squash/aaa-bad", s.f.base.as_str()]);
    let before = s.observe();
    let r = s.run("would-merge", Some(s.f.base.as_str()), 1, "refused");
    assert!(r.lines().contains("carries no Mortar-")); assert!(r.lines().contains(s.f.base.as_str()));
    assert_eq!(s.observe(), before);
    s.run("would-merge", Some(s.squash.change_id.as_str()), 0, "merges");
    assert_eq!(s.observe(), before);
    raw(&s.f.git, &["update-ref", "-d", "refs/mortar/squash/core"]);
    let before = s.observe();
    s.run("would-merge", Some(s.squash.change_id.as_str()), 1, "refused");
    assert_eq!(s.observe(), before);
}

/// Enumerate every commit object, including unreachable ones: a new trial merge commit must fail this oracle.
fn commits(s: &Landing) -> Vec<String> {
    let objects = raw(&s.f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let mut commits: Vec<_> = std::str::from_utf8(&objects).unwrap().lines().filter(|line| line.starts_with("commit ")).map(str::to_owned).collect();
    commits.sort(); commits
}
