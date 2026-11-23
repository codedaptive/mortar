//! Partner-base joins through the public interface, with real Git and caller-held locks.
mod support;
mod recorded_base;
use support::*;
use join::{BaseJoinAnswer, JoinAnswer, PartnerMerge, RequestBase};
use oplog::{OpPhase, RefMove};
use std::{fs, path::PathBuf};

#[test]
fn partner_merge_is_deterministic_and_omits_ancestors() {
    let f = Fixture::new();
    let root = f.commit(&[("common", b"base\n")], None, "root\n");
    let zone = f.commit(&[("common", b"base\n"), ("zone", b"z\n")], Some(&root), "zone\n");
    let p1 = f.commit(&[("common", b"base\n"), ("one", b"1\n")], Some(&root), "p1\n");
    let p2 = f.commit(&[("common", b"base\n"), ("two", b"2\n")], Some(&root), "p2\n");
    f.update(ZONE_REF, &zone); f.update(P1_REF, &p1); f.update(P2_REF, &p2);
    let partners = vec![partner("p1", &p1), partner("p2", &p2)];
    let result = f.merge(&zone, &partners);
    assert_eq!(f.merge(&zone, &partners), result);
    assert_eq!(result.partners, partners);
    assert_eq!(f.git.commit_info(&result.base).unwrap().parents, vec![zone.clone(), p1.clone(), p2.clone()]);
    let zone_object = f.run(&["cat-file", "-p", zone.as_str()], None, false);
    let merge_object = f.run(&["cat-file", "-p", result.base.as_str()], None, false);
    let author = zone_object.lines().find_map(|s| s.strip_prefix("author ")).unwrap();
    assert_eq!(merge_object.lines().find_map(|s| s.strip_prefix("author ")), Some(author));
    assert_eq!(merge_object.lines().find_map(|s| s.strip_prefix("committer ")), Some(author));
    assert_eq!(f.git.commit_info(&result.base).unwrap().tree,
        f.tree(&[("common", b"base\n"), ("zone", b"z\n"), ("one", b"1\n"), ("two", b"2\n")]));
    let dropped = f.merge(&zone, &[partner("already", &root), partner("p1", &p1)]);
    assert_eq!(dropped.partners, vec![partner("p1", &p1)]);
    assert_eq!(f.git.commit_info(&dropped.base).unwrap().parents, vec![zone.clone(), p1]);
    assert_eq!(f.merge(&zone, &[partner("already", &root), partner("equal", &zone)]),
        RequestBase { base: zone, partners: vec![] });
}

#[test]
fn partner_merge_conflict_refuses_without_moving_refs_or_writing_state() {
    let f = Fixture::new();
    let root = f.commit(&[("shared", b"base\n")], None, "root\n");
    let zone = f.commit(&[("shared", b"zone\n")], Some(&root), "zone\n");
    let p1 = f.commit(&[("shared", b"partner\n")], Some(&root), "p1\n");
    f.update(ZONE_REF, &zone); f.update(P1_REF, &p1);
    let before = f.refs();
    assert_eq!(join::make_partner_merge(&f.git, &zone, &[partner("p1", &p1)]).unwrap(),
        PartnerMerge::Conflicted { zone: "p1".into(), files: vec![PathBuf::from("shared")] });
    assert_eq!(f.refs(), before); assert!(!f.state_exists());
}

#[test]
fn unchanged_base_joins_own_commits_to_zone_in_one_operation() {
    let s = Scenario::new(false);
    let before = oplog::read_op_log(&s.f.repo.path.join(".git")).unwrap().records;
    let BaseJoinAnswer::ZoneJoined { zone_ref, request_ref, from, request_from, to } = s.f.join(&s.base)
        else { panic!("expected ZoneJoined") };
    assert_eq!(zone_ref, ZONE_REF); assert_eq!(request_ref, REQUEST_REF);
    assert_eq!(from, s.zone); assert_eq!(request_from, s.request);
    assert_eq!(s.f.git.read_ref(ZONE_REF).unwrap(), Some(to.clone()));
    assert_eq!(s.f.git.read_ref(REQUEST_REF).unwrap(), Some(to.clone()));
    let info = s.f.git.commit_info(&to).unwrap();
    assert_eq!(info.parents, vec![s.zone.clone()]);
    assert!(!s.f.git.is_ancestor(&s.base.base, &to).unwrap());
    assert_eq!(info.tree, s.f.tree(&[("shared", b"base\n"), ("zone", b"own\n")]));
    let records = oplog::read_op_log(&s.f.repo.path.join(".git")).unwrap().records;
    let done: Vec<_> = records[before.len()..].iter().filter(|r| r.phase == OpPhase::Done).collect();
    assert_eq!(done.len(), 1); assert_eq!(done[0].kind, "join-zone-request");
    assert_eq!(done[0].moves, vec![
        RefMove { refname: ZONE_REF.into(), old: Some(s.zone.clone()), new: to.clone() },
        RefMove { refname: REQUEST_REF.into(), old: Some(s.request.clone()), new: to.clone() },
    ]);
    assert_eq!(s.f.join(&s.base), BaseJoinAnswer::Plain(JoinAnswer::Joined { zone_ref: ZONE_REF.into(), request_tip: to }));
    assert_eq!(oplog::read_op_log(&s.f.repo.path.join(".git")).unwrap().records.len(), records.len());
}

#[test]
fn moved_partner_rebases_or_refuses_conflicting_partners() {
    for conflict in [false, true] {
        let s = Scenario::new(false);
        let zone = if conflict {
            let zone = s.f.commit(&[("shared", b"zone change\n"), ("zone", b"zone\n")], Some(&s.zone), "zone advance\n");
            s.f.update(ZONE_REF, &zone); zone
        } else { s.zone.clone() };
        let shared: &[u8] = if conflict { b"partner change\n" } else { b"base\n" };
        let current = s.f.commit(&[("shared", shared), ("partner", b"p1 advanced\n")], Some(&s.p1), "partner advance\n");
        s.f.update(P1_REF, &current);
        let before = s.f.refs();
        let answer = s.f.join(&s.base);
        if conflict {
            assert_eq!(answer, BaseJoinAnswer::PartnersConflicted { zone: "p1".into(), files: vec![PathBuf::from("shared")] });
            assert_eq!(s.f.refs(), before); assert!(!s.f.state_exists());
        } else {
            let BaseJoinAnswer::Rebased { request_ref, from, to, base } = answer else { panic!("expected Rebased") };
            assert_eq!(request_ref, REQUEST_REF); assert_eq!(from, s.request);
            assert_eq!(base.partners, vec![partner("p1", &current)]);
            assert_eq!(s.f.git.commit_info(&base.base).unwrap().parents, vec![zone.clone(), current]);
            let replayed = s.f.git.commit_info(&to).unwrap();
            assert_eq!(replayed.parents, vec![base.base]);
            assert_eq!(replayed.tree, s.f.tree(&[("shared", b"base\n"), ("zone", b"own\n"), ("partner", b"p1 advanced\n")]));
            assert_eq!(s.f.git.read_ref(REQUEST_REF).unwrap(), Some(to));
            assert_eq!(s.f.git.read_ref(ZONE_REF).unwrap(), Some(zone));
        }
    }
}

#[test]
fn moved_partner_replay_conflict_resolves_with_new_base() {
    let s = Scenario::new(true);
    let current = s.f.commit(&[("shared", b"partner change\n"), ("partner", b"p1\n")], Some(&s.p1), "partner advance\n");
    s.f.update(P1_REF, &current);
    let merge_tree = s.f.tree(&[("shared", b"partner change\n"), ("zone", b"zone\n"), ("partner", b"p1\n")]);
    let message = format!("mortar partner merge onto {}\n\npartner p1 {}\n", s.zone, current);
    let opts = gitio::RunOpts {
        stdin: Some(message.into_bytes()),
        env: vec![("GIT_COMMITTER_NAME".into(), "Author".into()),
                  ("GIT_COMMITTER_EMAIL".into(), "author@example.invalid".into()),
                  ("GIT_COMMITTER_DATE".into(), "1700000000 +0530".into())],
        ..gitio::RunOpts::default()
    };
    let output = s.f.git.run(&["commit-tree", merge_tree.as_str(), "-p", s.zone.as_str(), "-p", current.as_str()], &opts).unwrap();
    let expected = RequestBase { base: gitio::Oid::parse(String::from_utf8(output.stdout).unwrap().trim()).unwrap(),
        partners: vec![partner("p1", &current)] };
    let before = s.f.refs();
    assert!(matches!(s.f.join(&s.base), BaseJoinAnswer::Plain(JoinAnswer::Conflicted { commit, files, .. })
        if commit == s.request && files == vec![PathBuf::from("shared")]));
    assert_eq!(s.f.refs(), before);
    let tree = s.f.tree(&[("shared", b"resolved\n"), ("zone", b"own\n"), ("partner", b"p1\n")]);
    let first = s.f.caller(|| join::resolve_on_base(&s.f.git, &s.f.repo.path, &tree, ACTOR));
    assert!(matches!(first, (JoinAnswer::FilesWritten { .. }, None)));
    fs::write(s.f.repo.path.join("shared"), b"resolved\n").unwrap();
    let (answer, base) = s.f.caller(|| join::resolve_on_base(&s.f.git, &s.f.repo.path, &tree, ACTOR));
    let JoinAnswer::Rebased { request_ref, from, to, .. } = answer else { panic!("expected resolved rebase") };
    assert_eq!(request_ref, REQUEST_REF); assert_eq!(from, s.request); assert_eq!(base, Some(expected.clone()));
    let replayed = s.f.git.commit_info(&to).unwrap();
    assert_eq!(replayed.parents, vec![expected.base]);
    assert_eq!(replayed.tree, s.f.tree(&[("shared", b"resolved\n"), ("zone", b"own\n"), ("partner", b"p1\n")]));
    assert_eq!(s.f.git.read_ref(REQUEST_REF).unwrap(), Some(to));
    assert_eq!(s.f.git.read_ref(ZONE_REF).unwrap(), Some(s.zone));
}

#[test]
fn unchanged_base_replay_conflict_resolves_onto_the_zone_tip() {
    let f = Fixture::new();
    let root = f.commit(&[("shared", b"base\n")], None, "root\n");
    let zone = f.commit(&[("shared", b"base\n"), ("zone", b"zone\n")], Some(&root), "zone\n");
    let p1 = f.commit(&[("shared", b"base\n"), ("partner", b"partner\n")], Some(&root), "partner\n");
    f.update(P1_REF, &p1);
    let base = f.merge(&zone, &[partner("p1", &p1)]);
    let request = f.commit(&[("shared", b"base\n"), ("zone", b"zone\n"), ("partner", b"request edit\n")],
        Some(&base.base), "own\n");
    f.checkout(&zone, &request);
    let refs = f.refs();
    assert!(matches!(f.join(&base), BaseJoinAnswer::Plain(JoinAnswer::Conflicted { commit, files, .. })
        if commit == request && files == vec![PathBuf::from("partner")]));
    assert_eq!(f.refs(), refs);
    let tree = f.tree(&[("shared", b"base\n"), ("zone", b"zone\n"), ("partner", b"resolved\n")]);
    let first = f.caller(|| join::resolve_on_base(&f.git, &f.repo.path, &tree, ACTOR));
    assert!(matches!(first, (JoinAnswer::FilesWritten { .. }, None)));
    fs::write(f.repo.path.join("partner"), b"resolved\n").unwrap();
    let (answer, answered_base) = f.caller(|| join::resolve_on_base(&f.git, &f.repo.path, &tree, ACTOR));
    let JoinAnswer::Rebased { request_ref, from, to, .. } = answer else { panic!("expected resolved rebase") };
    assert_eq!(request_ref, REQUEST_REF); assert_eq!(from, request);
    assert_eq!(f.git.read_ref(ZONE_REF).unwrap(), Some(zone.clone()));
    assert_eq!(f.git.read_ref(REQUEST_REF).unwrap(), Some(to.clone()));
    assert_eq!(f.git.commit_info(&to).unwrap().parents, vec![zone.clone()]);
    assert_eq!(f.git.commit_info(&to).unwrap().tree,
        f.tree(&[("shared", b"base\n"), ("zone", b"zone\n"), ("partner", b"resolved\n")]));
    assert_eq!(answered_base, Some(RequestBase { base: zone, partners: vec![] }));
}

#[test]
fn no_partners_returns_the_plain_join_fast_forward_answer() {
    let f = Fixture::new();
    let zone = f.commit(&[("file", b"zone\n")], None, "zone\n");
    let request = f.commit(&[("file", b"request\n")], Some(&zone), "request\n");
    f.checkout(&zone, &request);
    let expected = JoinAnswer::FastForwarded { zone_ref: ZONE_REF.into(), from: zone.clone(), to: request.clone() };
    let actual = f.join(&RequestBase { base: zone.clone(), partners: vec![] });
    assert_eq!(actual, BaseJoinAnswer::Plain(expected.clone()));
    assert_eq!(f.git.read_ref(ZONE_REF).unwrap(), Some(request.clone()));
    let other = Fixture::new();
    let other_zone = other.commit(&[("file", b"zone\n")], None, "zone\n");
    let other_request = other.commit(&[("file", b"request\n")], Some(&other_zone), "request\n");
    other.checkout(&other_zone, &other_request);
    assert_eq!(other.caller(|| join::join(&other.git, &other.repo.path, ZONE, REQUEST_REF, ACTOR)), expected);
}
