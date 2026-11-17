//! run_need: a wrong argument count is need's usage (exit 4), answered at run_need.rs's argument check before any
//! lock, folders::need or git write. A missing zone branch is need's refusal (exit 1, missing_branch_report's
//! wording) before need writes its record, cone or refs. need's next step reads only what status_next needs, so an
//! unreadable record of another folder (read for status's overlaps) does not turn a completed need into a failure.
//! ZIT (S-119: the subject decides the group): the mortar binary runs as a child in a throwaway repository. The
//! RIT cases of need stay in tests/operational_rit/need.rs.
#[allow(dead_code)]
#[path = "cli_rit/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "cli_rit/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "operational_rit/observe.rs"]
mod observe;
#[allow(dead_code)]
#[path = "operational_rit/need_fixture.rs"]
mod need_fixture;

use need_fixture::Need;
use observe::{before, peer, record_path};
use support::{raw, ZONE};

#[test]
fn need_wrong_argument_count_is_usage_without_work() {
    let n = Need::new(); let old = before(&n.f, &n.folder);
    for args in [vec!["--json", "need"], vec!["--json", "need", "src", "sub"]] {
        let report = n.report(&args, 4, "usage"); assert!(report.lines().contains("need takes one path"));
        assert_eq!(before(&n.f, &n.folder), old);
    }
}

#[test]
fn need_with_a_missing_zone_branch_refuses_before_writing() {
    let n = Need::new(); raw(&n.f.git, &["update-ref", "-d", ZONE]);
    let paths = n.paths(); let cone = n.f.git.sparse_list(&n.folder).unwrap(); let old = before(&n.f, &n.folder);
    let report = n.report(&["--json", "need", "sub"], 1, "refused");
    assert_eq!(report.lines(), "the branch refs/heads/zone/core does not exist");
    assert_eq!(report.text("owner"), "the zone's coordinator");
    assert_eq!(n.paths(), paths, "need must not record the path when it refuses");
    assert_eq!(n.f.git.sparse_list(&n.folder).unwrap(), cone); assert!(!n.folder.join("sub/keep").exists());
    assert_eq!(before(&n.f, &n.folder), old);
}

#[test]
fn need_does_not_read_other_folders_for_its_next_step() {
    let n = Need::new();
    let other = peer(&n.f, "peer", &["src".into()]);
    std::fs::write(record_path(&n.f, &other), b"{not a folder record\n").unwrap();
    let report = n.report(&["--json", "need", "sub"], 0, "needed");
    // A fresh request folder: its branch is at its base and its tree is the tip's, so it reads saved with no work
    // yet, and status_next tells it to make its change (S-144, S-145).
    assert_eq!(report.text("next"), "make your change, then mr save");
    assert_eq!(report.text("path"), "sub"); assert_eq!(n.paths(), vec!["sub"]);
    assert_eq!(std::fs::read(n.folder.join("sub/keep")).unwrap(), b"keep\n");
}
