//! Joins the `mr would-merge` binary command with Land's squash lookup/merge check and real Git/Oplog locks.
use super::fixture::Landing;
use crate::support::raw;

#[test]
fn yes_by_full_commit_and_change_id_names_the_land_command_and_leaves_refs_log_folder_and_commit_inventory_unchanged() { would_merge_by_commit_and_change_id_moves_nothing(false); }

#[test]
fn no_by_full_commit_and_change_id_lists_the_conflicting_file_and_leaves_refs_log_folder_and_commit_inventory_unchanged() { would_merge_by_commit_and_change_id_moves_nothing(true); }

fn would_merge_by_commit_and_change_id_moves_nothing(conflict: bool) {
    let s = Landing::new(true);
    if conflict {
        let next = s.f.commit(&s.f.base, &[("src/file", b"origin work\n")], "origin work\n");
        raw(&s.f.git, &["update-ref", "refs/heads/main", next.as_str()]);
        raw(&s.f.git, &["update-ref", "refs/heads/integration", next.as_str()]);
        s.sync(&s.f.base, &next);
    }
    for word in [s.squash.commit.as_str(), s.squash.change_id.as_str()] {
        let before = s.observe(); let commits = commits(&s);
        let r = s.run("would-merge", Some(word), if conflict { 1 } else { 0 }, if conflict { "conflict" } else { "merges" });
        assert_eq!(r.text("change_id"), s.squash.change_id.as_str());
        assert_eq!(r.text("commit"), s.squash.commit.as_str());
        if conflict { assert_eq!(r.strings("files"), &["src/file"]); }
        else { assert_eq!(r.text("next"), format!("mr land {} --actor <name>", s.squash.commit)); }
        assert!(r.lines().contains("no ref moved and no commit was written"));
        assert_eq!(s.observe(), before); assert_eq!(self::commits(&s), commits);
    }
}

#[test]
fn missing_and_duplicate_change_ids_are_refused_without_movement_and_name_the_ambiguity() {
    let s = Landing::without_remote(true);
    let before = s.observe();
    let r = s.run("would-merge", Some("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), 3, "refused");
    assert!(r.lines().contains("no squash at refs/mortar/squash/* carries change ID"));
    assert!(r.text("next").contains("mr squash <zone>")); assert_eq!(s.observe(), before);
    raw(&s.f.git, &["update-ref", "refs/mortar/squash/duplicate", s.squash.commit.as_str()]);
    let before = s.observe();
    let r = s.run("would-merge", Some(s.squash.change_id.as_str()), 3, "refused");
    assert!(r.lines().contains("refs/mortar/squash/core, refs/mortar/squash/duplicate"));
    assert!(r.text("next").contains("mr would-merge <commit>")); assert_eq!(s.observe(), before);
    s.run("would-merge", Some(s.squash.commit.as_str()), 0, "merges");
    assert_eq!(s.observe(), before);
}

#[test]
fn a_commit_without_mortar_trailers_is_refused_but_a_bad_ref_beside_a_good_ref_is_skipped() {
    let s = Landing::without_remote(true);
    raw(&s.f.git, &["update-ref", "refs/mortar/squash/aaa-bad", s.f.base.as_str()]);
    let before = s.observe();
    let r = s.run("would-merge", Some(s.f.base.as_str()), 3, "refused");
    assert!(r.lines().contains("carries no Mortar-")); assert!(r.lines().contains(s.f.base.as_str()));
    assert_eq!(s.observe(), before);
    s.run("would-merge", Some(s.squash.change_id.as_str()), 0, "merges");
    assert_eq!(s.observe(), before);
    raw(&s.f.git, &["update-ref", "-d", "refs/mortar/squash/core"]);
    let before = s.observe();
    s.run("would-merge", Some(s.squash.change_id.as_str()), 3, "refused");
    assert_eq!(s.observe(), before);
}

#[test]
fn would_merge_savings_large_patch_and_conflict() {
    let s = Landing::new(true);
    let patch: String = (0..400).map(|i| format!("zone change {i} é\n")).collect();
    let zone = s.f.commit(&s.f.base, &[("src/file", patch.as_bytes())], "large zone\n");
    raw(&s.f.git, &["update-ref", "refs/heads/zone/core", zone.as_str()]);
    let land::SquashAnswer::Made(squash) = s.locked(|| land::squash(&s.f.git, &s.refs, "core", "savings-setup").unwrap()) else { panic!("squash") };
    let range = format!("refs/heads/integration..{}", squash.commit);
    let b = crate::run_rereview_savings_tests::git_count(&s.f.git.repo, &s.f.root, &s.f.git.env,
        &["--no-pager", "-c", "color.ui=never", "log", "-p", "--no-color", "--no-ext-diff", &range, "--"]);
    let output = s.f.command(&s.f.git.repo, &["--actor", "land-flow", "would-merge", squash.commit.as_str()]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert!(text.starts_with("[mr] tokens saved ≈ "), "{text}");
    let (line, body) = text.split_once("\n\n").unwrap();
    assert!(line.contains("git log -p <base>..<commit>"));
    assert!(line.strip_prefix("[mr] tokens saved ≈ ").unwrap().split_whitespace().next().unwrap().replace(',', "").parse::<usize>().unwrap() > 0);
    assert!(b > body.chars().count());
    let output = s.f.command(&s.f.git.repo, &["--json", "--actor", "land-flow", "would-merge", squash.commit.as_str()]).output().unwrap();
    let json = std::str::from_utf8(&output.stdout).unwrap();
    assert!(json.ends_with(&format!("\"savings\":{{\"baseline\":\"git log -p <base>..<commit>\",\"baselineChars\":{b},\"outputChars\":{}}}}}\n", body.chars().count())), "{json}");
    let log = std::fs::read_to_string(s.f.root.join(".local/state/mortar/savings.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 2);
    for line in log.lines() { assert!(line.contains(&format!("\"verb\":\"would-merge\",\"output_tokens\":{},\"baseline_tokens\":{}", (body.chars().count()+2)/4, (b+2)/4)), "{line}"); }
    let next = s.f.commit(&s.f.base, &[("src/file", b"conflicting work\n")], "conflict\n");
    raw(&s.f.git, &["update-ref", "refs/heads/integration", next.as_str()]);
    for json in [false, true] {
        let mut cmd = s.f.command(&s.f.git.repo, &[]);
        if json { cmd.arg("--json"); }
        let out = cmd.args(["--actor", "land-flow", "would-merge", squash.commit.as_str()]).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(!text.contains("[mr] tokens saved") && !text.contains("\"savings\""), "{text}");
    }
    assert_eq!(std::fs::read_to_string(s.f.root.join(".local/state/mortar/savings.jsonl")).unwrap(), log);
}

/// Enumerate every commit object, including unreachable ones: a new trial merge commit must fail this oracle.
fn commits(s: &Landing) -> Vec<String> {
    let objects = raw(&s.f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let mut commits: Vec<_> = std::str::from_utf8(&objects).unwrap().lines().filter(|line| line.starts_with("commit ")).map(str::to_owned).collect();
    commits.sort(); commits
}
