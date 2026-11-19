//! bisect on throwaway repositories: five landings after a passing L0, the third breaks check.sh.
//! Each case checks bisect's answer and that no probe worktree, git bisect ref or BISECT_* file is left behind.

#[path = "bisect_zit/history.rs"]
mod history;

use std::time::Duration;

use history::{build, git_of, off_main, run, History, Third, IDS};
use land::{bisect, BisectAnswer, BisectError, BisectReport, Culprit, LandRefs, ProbeOutcome, TestCommand};
use oplog::ChangeId;

const LIMIT: Duration = Duration::from_secs(60);

fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

fn check_script() -> TestCommand {
    TestCommand { program: "/bin/sh".into(), args: vec!["check.sh".into()] }
}

/// Runs bisect from `landings[last_pass]` to L5 with `command`.
fn bisect_in(history: &History, last_pass: usize, command: &TestCommand) -> Result<BisectReport, BisectError> {
    let git = git_of(&history.repo.path);
    bisect(&git, &refs(), &history.landings[last_pass], &history.landings[5], command, LIMIT)
}

/// The repository's own worktree is the only one, and no git bisect state was written.
fn assert_clean(history: &History) {
    let path = &history.repo.path;
    let list = run(path, &["worktree", "list", "--porcelain"]);
    assert_eq!(list.lines().filter(|line| line.starts_with("worktree ")).count(), 1, "{list}");
    assert_eq!(run(path, &["for-each-ref", "refs/bisect"]), "");
    let bisect_files = std::fs::read_dir(path.join(".git"))
        .unwrap()
        .filter(|entry| entry.as_ref().unwrap().file_name().to_string_lossy().starts_with("BISECT_"))
        .count();
    assert_eq!(bisect_files, 0);
}

/// Every probe's folder is gone, and the repository is clean.
fn assert_probes_removed(history: &History, report: &BisectReport) {
    assert!(!report.probes.is_empty());
    for probe in &report.probes {
        assert!(!probe.folder.exists(), "{} still exists", probe.folder.display());
    }
    assert_clean(history);
}

#[test]
fn bisect_names_the_merge_landing_that_broke_the_script() {
    let history = build(Third::Merge("exit 1\n"), None);
    let report = bisect_in(&history, 0, &check_script()).unwrap();
    let expected = Culprit {
        commit: history.landings[3].clone(),
        change_id: Some(ChangeId::parse(IDS[2]).unwrap()),
        zone: Some("zone-three".into()),
    };
    assert_eq!(report.answer, BisectAnswer::Culprit(expected));
    assert_probes_removed(&history, &report);
}

#[test]
fn a_timed_out_probe_counts_as_a_fail() {
    let history = build(Third::Merge("exec sleep 600\n"), Some("exit 1\n"));
    let report = bisect_in(&history, 0, &check_script()).unwrap();
    let BisectAnswer::Culprit(culprit) = &report.answer else { panic!("no culprit: {report}") };
    assert_eq!(culprit.commit, history.landings[3]);
    let timed_out: Vec<_> =
        report.probes.iter().filter(|probe| matches!(probe.outcome, ProbeOutcome::TimedOut { .. })).collect();
    assert_eq!(timed_out.len(), 1, "{report}");
    assert_eq!(timed_out[0].outcome, ProbeOutcome::TimedOut { limit: LIMIT });
    assert_eq!(timed_out[0].commit, history.landings[3]);
    assert!(report.to_string().contains("timed out"), "{report}");
    assert_probes_removed(&history, &report);
}

#[test]
fn a_command_that_cannot_start_stops_the_search() {
    let history = build(Third::Merge("exit 1\n"), None);
    let command = TestCommand { program: "/nonexistent/mortar-no-such-program".into(), args: vec![] };
    let result = bisect_in(&history, 0, &command);
    let Err(BisectError::ProbeCouldNotRun { commit, .. }) = &result else {
        panic!("not ProbeCouldNotRun: {result:?}")
    };
    assert_eq!(*commit, history.landings[5]);
    assert_clean(&history);
}

#[test]
fn a_last_pass_off_the_first_parent_line_is_refused() {
    let history = build(Third::Merge("exit 1\n"), None);
    let squash = history.squash.clone().unwrap();
    let git = git_of(&history.repo.path);
    let result = bisect(&git, &refs(), &squash, &history.landings[5], &check_script(), LIMIT);
    let Err(BisectError::OffTheLine { last_pass, failed }) = &result else { panic!("not OffTheLine: {result:?}") };
    assert_eq!((last_pass, failed), (&squash, &history.landings[5]));
    assert_clean(&history);
}

#[test]
fn a_failed_commit_off_origin_is_refused() {
    let history = build(Third::Merge("exit 1\n"), None);
    let side = off_main(&history);
    let git = git_of(&history.repo.path);
    let result = bisect(&git, &refs(), &history.landings[0], &side, &check_script(), LIMIT);
    let Err(BisectError::NotOnOrigin { failed, origin_branch }) = &result else {
        panic!("not NotOnOrigin: {result:?}")
    };
    assert_eq!((failed, origin_branch.as_str()), (&side, "main"));
    assert_clean(&history);
}

#[test]
fn a_landing_without_trailers_is_still_named() {
    let history = build(Third::PlainFastForward("exit 1\n"), None);
    let report = bisect_in(&history, 0, &check_script()).unwrap();
    let expected = Culprit { commit: history.landings[3].clone(), change_id: None, zone: None };
    assert_eq!(report.answer, BisectAnswer::Culprit(expected));
    assert!(report.to_string().contains("carries no squash trailers"), "{report}");
    assert_probes_removed(&history, &report);
}
