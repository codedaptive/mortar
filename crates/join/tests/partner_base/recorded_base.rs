use super::support::*;
use gitio::Oid;
use join::{BaseJoinAnswer, JoinAnswer, RecordedBase, RequestBase};
use oplog::{ApplyOutcome, OpCaller, OpPhase, ValuedRecord};
use std::fs;

fn done(f: &Fixture, kind: &str) -> ValuedRecord {
    oplog::read_valued_log(&f.repo.path.join(".git")).unwrap().records.into_iter()
        .rev().find(|op| op.record.kind == kind && op.record.phase == OpPhase::Done).unwrap()
}

fn check_and_undo(f: &Fixture, kind: &str, before: RequestBase, after: RequestBase) {
    let op = done(f, kind);
    let expected = RecordedBase::Recorded { before, after };
    assert_eq!(join::recorded_base(&op), expected);
    let caller = OpCaller { key: "undo-recorded-base".into(), actor: ACTOR.into(), reason: "test undo".into() };
    let outcome = f.caller(|| oplog::undo(&f.git, &f.repo.path.join(".git"), &[REQUEST_REF.into()], &caller, &[])
        .map_err(join::JoinRunError::Oplog));
    assert!(matches!(outcome, ApplyOutcome::Done(_)));
    for movement in &op.record.moves {
        assert_eq!(f.git.read_ref(&movement.refname).unwrap(), movement.old);
    }
    let original = oplog::read_valued_log(&f.repo.path.join(".git")).unwrap().records.into_iter()
        .find(|entry| entry.record.kind == kind && entry.record.phase == OpPhase::Done
            && entry.record.op_id == op.record.op_id).unwrap();
    assert_eq!(join::recorded_base(&original), expected);
}

fn plain_scenario(conflict: bool) -> (Fixture, RequestBase, Oid) {
    let f = Fixture::new();
    let root = f.commit(&[("shared", b"base\n")], None, "root\n");
    let zone_shared: &[u8] = if conflict { b"zone\n" } else { b"base\n" };
    let zone = f.commit(&[("shared", zone_shared), ("zone", b"zone\n")], Some(&root), "zone\n");
    let request_shared: &[u8] = if conflict { b"request\n" } else { b"base\n" };
    let request = f.commit(&[("shared", request_shared), ("own", b"own\n")], Some(&root), "request\n");
    f.checkout(&zone, &request);
    (f, RequestBase { base: root, partners: vec![] }, zone)
}

#[test]
fn partner_rebase_records_bases_and_survives_undo() {
    let s = Scenario::new(false);
    let current = s.f.commit(&[("shared", b"base\n"), ("partner", b"advanced\n")], Some(&s.p1), "advance\n");
    s.f.update(P1_REF, &current);
    let expected = s.f.merge(&s.zone, &[partner("p1", &current)]);
    let BaseJoinAnswer::Rebased { base, .. } = s.f.join(&s.base) else { panic!("expected partner rebase") };
    assert_eq!(base, expected);
    check_and_undo(&s.f, "join-rebase", s.base, expected);
}

#[test]
fn plain_rebase_records_bases_and_survives_undo() {
    let (f, before, zone) = plain_scenario(false);
    assert!(matches!(f.join(&before), BaseJoinAnswer::Plain(JoinAnswer::Rebased { .. })));
    check_and_undo(&f, "join-rebase", before, RequestBase { base: zone, partners: vec![] });
}

#[test]
fn zone_join_records_bases_and_survives_undo() {
    let s = Scenario::new(false);
    assert!(matches!(s.f.join(&s.base), BaseJoinAnswer::ZoneJoined { .. }));
    check_and_undo(&s.f, "join-zone-request", s.base, RequestBase { base: s.zone, partners: vec![] });
}

fn finish_conflict(f: &Fixture, tree: &Oid, after: &RequestBase) {
    assert!(matches!(f.caller(|| join::resolve_on_base(&f.git, &f.repo.path, tree, ACTOR)),
        (JoinAnswer::FilesWritten { .. }, None)));
    fs::write(f.repo.path.join("shared"), b"resolved\n").unwrap();
    let (answer, base) = f.caller(|| join::resolve_on_base(&f.git, &f.repo.path, tree, ACTOR));
    assert!(matches!(answer, JoinAnswer::Rebased { .. }));
    assert_eq!(base, Some(after.clone()));
    let op = done(f, "join-rebase");
    assert_eq!(op.values.iter().take(2).map(|(key, _)| key.as_str()).collect::<Vec<_>>(),
        vec!["folder_tree", "join_state"]);
}

#[test]
fn partner_resolve_records_bases_and_survives_undo() {
    let s = Scenario::new(true);
    let current = s.f.commit(&[("shared", b"partner change\n"), ("partner", b"p1\n")], Some(&s.p1), "advance\n");
    s.f.update(P1_REF, &current);
    let after = s.f.merge(&s.zone, &[partner("p1", &current)]);
    assert!(matches!(s.f.join(&s.base), BaseJoinAnswer::Plain(JoinAnswer::Conflicted { .. })));
    let tree = s.f.tree(&[("shared", b"resolved\n"), ("zone", b"own\n"), ("partner", b"p1\n")]);
    finish_conflict(&s.f, &tree, &after);
    check_and_undo(&s.f, "join-rebase", s.base, after);
}

#[test]
fn plain_resolve_records_bases_and_survives_undo() {
    let (f, before, zone) = plain_scenario(true);
    assert!(matches!(f.join(&before), BaseJoinAnswer::Plain(JoinAnswer::Conflicted { .. })));
    let tree = f.tree(&[("shared", b"resolved\n"), ("own", b"own\n")]);
    let after = RequestBase { base: zone, partners: vec![] };
    finish_conflict(&f, &tree, &after);
    check_and_undo(&f, "join-rebase", before, after);
}

#[test]
fn plain_join_without_known_base_is_not_recorded() {
    let (f, _, _) = plain_scenario(false);
    assert!(matches!(f.caller(|| join::join(&f.git, &f.repo.path, ZONE, REQUEST_REF, ACTOR)),
        JoinAnswer::Rebased { .. }));
    assert_eq!(join::recorded_base(&done(&f, "join-rebase")), RecordedBase::NotRecorded);
}
