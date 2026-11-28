// The not-UTF-8 hook input case is in run_guard_hook_tests.rs.
use crate::{cid::CID, guard_fixture::Guard, json::check, refs::refs};
use land::{BatchLandAnswer, PublishAnswer, Squash};
use oplog::{KillPidProbe, LockName, Op, OpPhase, ProtectedRef, RefMove};
use std::fs;

#[test]
fn guard_installed_hook_allows_unguarded_ref_without_stdout_or_stderr() {
    // actual Git transaction -> installed mortar hook -> Land guard/Oplog -> binary silent allow.
    let g = Guard::new();
    let output = g.output(&["update-ref", "refs/heads/free", g.new.as_str()]);
    assert!(output.status.success(), "{output:?}"); assert!(output.stdout.is_empty()); assert!(output.stderr.is_empty());
    assert_eq!(g.f.git.read_ref("refs/heads/free").unwrap(), Some(g.new.clone()));
}

#[test]
fn guard_allow_is_silent_in_plain_and_json_without_actor_or_folder() {
    // Binary output boundary: even --json has ZERO bytes, not an empty JSON report.
    let g = Guard::new(); let before = g.branches();
    let input = format!("{} {} refs/heads/free\n", g.f.base, g.new);
    for args in [vec!["guard-hook", "reference-transaction", "prepared"],
        vec!["--json", "guard-hook", "reference-transaction", "prepared"]] {
        let output = g.hook(&args, input.as_bytes());
        assert_eq!(output.status.code(), Some(0), "{output:?}"); assert!(output.stdout.is_empty()); assert!(output.stderr.is_empty());
    }
    assert_eq!(g.branches(), before);
    let actor = check(g.hook(&["--json", "guard-hook", "--actor", "intruder", "reference-transaction", "prepared"], b""),
        "guard-hook", 2, "usage");
    assert!(actor.lines().contains("take no --actor"));
}

#[test]
fn guard_refuses_direct_updates_and_unknown_op_tokens_on_both_protected_refs() {
    for branch in ["refs/heads/main", "refs/heads/integration"] {
        let g = Guard::new(); let before = g.branches();
        for token in [None, Some("no-such-open-intent")] {
            let mut cmd = g.command(&["update-ref", branch, g.new.as_str(), g.f.base.as_str()]);
            if let Some(token) = token { cmd.env(oplog::MORTAR_OP_ENV, token); }
            g.no_move(&before, cmd.output().unwrap(), &format!("only Mortar moves {branch}"));
        }
    }
}

#[test]
fn guard_refuses_actual_commit_against_a_protected_branch() {
    let g = Guard::new();
    fs::write(g.f.git.repo.join("src/file"), b"unauthorized staged commit\n").unwrap();
    assert!(g.output(&["add", "src/file"]).status.success());
    let before = g.branches();
    g.no_move(&before, g.output(&["commit", "-m", "must not move main"]), "only Mortar moves refs/heads/main");
    assert_eq!(fs::read(g.f.git.repo.join("src/file")).unwrap(), b"unauthorized staged commit\n");
}

#[test]
fn guard_refuses_push_by_guarded_name_or_url_but_allows_a_distinct_remote() {
    // actual pre-push hook, actual two disposable bare remotes; no network service.
    let g = Guard::new(); let before = g.branches(); let spec = format!("{}:refs/heads/main", g.new);
    for target in ["origin", g.origin.to_str().unwrap()] {
        g.no_move(&before, g.output(&["push", target, &spec]), "only Mortar pushes refs/heads/main to origin");
    }
    let output = g.output(&["push", "other", &spec]); assert!(output.status.success(), "{output:?}");
    assert_eq!(g.remote(true), Some(g.new.clone())); assert_eq!(g.remote(false), Some(g.f.base.clone()));
    // A successful push legitimately creates the local tracking ref; protected local heads still stay put.
    let mut expected = before;
    expected.extend_from_slice(format!("refs/remotes/other/main {}\n", g.new).as_bytes());
    assert_eq!(g.branches(), expected);
}

#[test]
fn guard_unreadable_guarded_push_urls_fail_closed_for_another_remote_name() {
    let g = Guard::new();
    // Removing only remote.origin.url can leave a configured remote with a fallback URL. Remove the remote
    // itself, leaving guard.json's guarded name intact, and independently verify the REAL query fails.
    assert!(g.output(&["remote", "remove", "origin"]).status.success());
    let query = g.output(&["remote", "get-url", "--push", "--all", "--end-of-options", "origin"]);
    assert!(!query.status.success(), "URL-read fault must actually exist: {query:?}");
    let before = g.branches(); let spec = format!("{}:refs/heads/main", g.new);
    g.no_move(&before, g.output(&["push", "other", &spec]), "cannot read origin's push URLs");
    assert_eq!(g.remote(true), Some(g.f.base.clone()));
}

#[test]
fn guard_missing_or_broken_config_fails_closed_naming_the_installer() {
    for broken in [false, true] {
        let g = Guard::new(); let path = g.f.common.join("mortar/guard.json");
        if broken { fs::write(&path, b"{").unwrap(); } else { fs::remove_file(&path).unwrap(); }
        let before = g.branches(); let output = g.output(&["update-ref", "refs/heads/main", g.new.as_str()]);
        assert!(!output.status.success());
        let reason = String::from_utf8_lossy(&output.stderr);
        assert!(reason.contains("guard.json")); assert!(reason.contains("the harness that installs the guard"));
        assert!(reason.contains("mr guard install --actor")); assert_eq!(g.branches(), before);
        let input = format!("{} {} refs/heads/main\n", g.f.base, g.new);
        for json in [true, false] {
            let args = if json { vec!["--json", "guard-hook", "reference-transaction", "prepared"] }
                else { vec!["guard-hook", "reference-transaction", "prepared"] };
            let output = g.hook(&args, input.as_bytes());
            if json {
                let report = check(output, "guard-hook", 2, if broken { "bad-config" } else { "no-guard-config" });
                assert_eq!(report.text("owner"), "the harness that installs the guard");
                assert!(report.lines().contains("guard.json")); assert!(report.text("next").contains("mr guard install --actor"));
            } else {
                assert_eq!(output.status.code(), Some(2), "{output:?}");
                let text = String::from_utf8(output.stdout).unwrap();
                assert!(text.contains("guard.json")); assert!(text.contains("the harness that installs the guard"));
                assert!(text.contains("mr guard install --actor"));
            }
            assert_eq!(g.branches(), before);
        }
    }
}

#[test]
fn guard_public_apply_protected_uses_its_actual_open_intent_and_finishes_once() {
    let g = Guard::new();
    let op = Op { op_id: "guard-apply-1".into(), key: "guard-apply-key".into(), kind: "guard-validation".into(),
        actor: "guard-validation".into(), reason: "permitted main fast-forward".into(), at: 7,
        moves: vec![RefMove { refname: "refs/heads/main".into(), old: Some(g.f.base.clone()), new: g.new.clone() }] };
    oplog::with_lock(&g.f.git, &g.f.common, &LockName::Land, "guard-apply", &KillPidProbe, || {
        oplog::recover(&g.f.git, &g.f.common)?;
        oplog::apply_protected(&g.f.git, &g.f.common, &op,
            &[ProtectedRef { refname: "refs/heads/main".into(), by: "landing".into() }])
    }).unwrap();
    assert_eq!(g.f.git.read_ref("refs/heads/main").unwrap(), Some(g.new.clone()));
    let records = g.f.records(); let records: Vec<_> = records.iter().filter(|r| r.op_id == op.op_id).collect();
    assert_eq!(records.len(), 2); assert_eq!(records[0].phase, OpPhase::Intent); assert_eq!(records[1].phase, OpPhase::Done);
    for record in records { assert_eq!(record.moves, op.moves); }
}

#[test]
fn guard_public_land_and_publish_push_pass_the_installed_cli_hooks() {
    // Land -> protected local transaction -> marked remote push -> installed CLI hooks -> real remote.
    let g = Guard::new();
    let squash = Squash { commit: g.new.clone(), change_id: oplog::ChangeId::parse(CID).unwrap(),
        source: g.new.clone(), requests: vec![] };
    oplog::with_lock(&g.f.git, &g.f.common, &LockName::Land, "guard-publish", &KillPidProbe, || {
        oplog::recover(&g.f.git, &g.f.common).unwrap();
        assert_eq!(land::land_batch(&g.f.git, &refs(), &squash, "guard-validation").unwrap(), BatchLandAnswer::Landed { integration: g.new.clone() });
        assert_eq!(land::publish(&g.f.git, &refs(), "guard-validation").unwrap(), PublishAnswer::Published { origin: g.new.clone() });
        Ok::<_, std::io::Error>(())
    }).unwrap();
    assert_eq!(g.remote(false), Some(g.new.clone()));
    assert_eq!(g.f.git.read_ref("refs/heads/main").unwrap(), Some(g.new.clone()));
    let records = oplog::read_marked_log(&g.f.common).unwrap().records;
    let push: Vec<_> = records.iter().filter(|r| r.record.kind == "publish-push").collect();
    assert_eq!(push.len(), 2); assert_eq!(push[0].record.phase, OpPhase::Intent); assert_eq!(push[1].record.phase, OpPhase::Done);
    for marked in push { assert_eq!(marked.remote.as_deref(), Some("origin")); }
}
