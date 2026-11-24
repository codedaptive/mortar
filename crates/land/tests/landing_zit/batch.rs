use crate::fixture::{refs, tree, Fixture, ACTOR, ID_A, ID_B};
use gitio::Oid;
use land::{back_out_batch, land, land_batch, read_landing_batch, BatchBackOutAnswer, BatchLandAnswer, BatchMember, LandAnswer, Squash};
use oplog::{oplog_path, read_op_log, ChangeId, OpPhase};
use std::path::PathBuf;

const ID_C: &str = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";

fn squash(fx: &Fixture, zone: &str, id: &str, file: &str, content: &str) -> Squash {
    let message = format!("{zone}: squash\n\nMortar-Change-Id: {id}\nMortar-Zone: {zone}\nMortar-Request: r1\n");
    Squash {
        commit: fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), (file, content)]),
            std::slice::from_ref(&fx.base), &message, None, None).unwrap(),
        change_id: ChangeId::parse(id).unwrap(), source: fx.base.clone(), requests: vec!["r1".into()],
    }
}

fn landed(fx: &Fixture, squash: &Squash) -> Oid {
    match land_batch(&fx.git, &refs(), squash, ACTOR).unwrap() {
        BatchLandAnswer::Landed { integration } => integration,
        answer => panic!("expected Landed, got {answer:?}"),
    }
}

fn state(fx: &Fixture, members: &[(&str, &str, &Squash, &Oid)]) -> String {
    let entries: Vec<String> = members.iter().map(|(id, zone, squash, integration)|
        format!("{{\"change_id\":\"{id}\",\"zone\":\"{zone}\",\"squash\":\"{}\",\"integration\":\"{integration}\"}}", squash.commit)).collect();
    format!("{{\"origin_at_start\":\"{}\",\"landings\":[{}]}}\n", fx.base, entries.join(","))
}

fn seed(fx: &Fixture, members: &[(&str, &str, &Squash, &Oid)]) {
    std::fs::create_dir_all(fx.common.join("mortar")).unwrap();
    std::fs::write(fx.common.join("mortar/landing.json"), state(fx, members)).unwrap();
    fx.set_branch("integration", members.last().unwrap().3);
}

#[test]
fn read_landing_batch_names_the_members_in_landing_order() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    let first = BatchMember { change_id: ChangeId::parse(ID_A).unwrap(), zone: "alpha".into() };
    let second = BatchMember { change_id: ChangeId::parse(ID_B).unwrap(), zone: "beta".into() };
    assert_eq!(read_landing_batch(&fx.git).unwrap(), None);
    assert_eq!(land_batch(&fx.git, &refs(), &a, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: a.commit.clone() });
    assert_eq!(read_landing_batch(&fx.git).unwrap(), Some(vec![first.clone()]));
    assert!(matches!(land_batch(&fx.git, &refs(), &b, ACTOR).unwrap(), BatchLandAnswer::Landed { .. }));
    assert_eq!(read_landing_batch(&fx.git).unwrap(), Some(vec![first, second]));
    assert_eq!(back_out_batch(&fx.git, &refs(), ACTOR).unwrap(), BatchBackOutAnswer::BackedOut {
        change_id: ChangeId::parse(ID_B).unwrap(),
        change_ids: vec![ChangeId::parse(ID_A).unwrap(), ChangeId::parse(ID_B).unwrap()],
    });
    assert_eq!(read_landing_batch(&fx.git).unwrap(), None);
}

#[test]
fn three_zones_append_in_order() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    let c = squash(&fx, "gamma", ID_C, "c1", "c\n");
    let ia = a.commit.clone();
    let ib = fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("a1", "a\n"), ("b1", "b\n")]),
        &[ia.clone(), b.commit.clone()], &format!("land {ID_B} (beta)\n"), None, None).unwrap();
    let ic = fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("a1", "a\n"), ("b1", "b\n"), ("c1", "c\n")]),
        &[ib.clone(), c.commit.clone()], &format!("land {ID_C} (gamma)\n"), None, None).unwrap();
    for (member, expected) in [(&a, &ia), (&b, &ib), (&c, &ic)] {
        assert_eq!(land_batch(&fx.git, &refs(), member, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: expected.clone() });
    }
    for (commit, parent, member, message) in [(&ib, &ia, &b, format!("land {ID_B} (beta)\n")), (&ic, &ib, &c, format!("land {ID_C} (gamma)\n"))] {
        let info = fx.git.commit_info(commit).unwrap();
        assert_eq!(info.parents, vec![parent.clone(), member.commit.clone()]);
        assert_eq!(info.message, message);
    }
    assert_eq!(fx.branch("integration"), ic);
    assert_eq!(fx.landing_json(), Some(state(&fx, &[(ID_A, "alpha", &a, &ia), (ID_B, "beta", &b, &ib), (ID_C, "gamma", &c, &ic)])));
}

#[test]
fn conflict_keeps_batch_and_next_zone_can_append() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "shared", "first\n");
    let b = squash(&fx, "beta", ID_B, "shared", "second\n");
    let c = squash(&fx, "gamma", ID_C, "other", "third\n");
    let ia = landed(&fx, &a);
    let before = fx.landing_json();
    let log = std::fs::read(oplog_path(&fx.common)).unwrap();
    assert_eq!(land_batch(&fx.git, &refs(), &b, ACTOR).unwrap(), BatchLandAnswer::Conflicted { files: vec![PathBuf::from("shared")] });
    assert_eq!(fx.branch("integration"), ia);
    assert_eq!(fx.landing_json(), before);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).unwrap(), log);
    let ic = landed(&fx, &c);
    assert_eq!(fx.landing_json(), Some(state(&fx, &[(ID_A, "alpha", &a, &ia), (ID_C, "gamma", &c, &ic)])));
}

#[test]
fn old_land_of_another_zone_at_the_tip_is_busy_and_writes_nothing() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    assert_eq!(land_batch(&fx.git, &refs(), &a, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: a.commit.clone() });
    let state = fx.landing_json();
    let log = std::fs::read(oplog_path(&fx.common)).unwrap();
    let all_refs = fx.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &gitio::RunOpts::default()).unwrap().stdout;
    assert_eq!(land(&fx.git, &refs(), &b, ACTOR).unwrap(), LandAnswer::Busy { change_id: ChangeId::parse(ID_A).unwrap() });
    assert_eq!(fx.branch("integration"), a.commit);
    assert_eq!(fx.landing_json(), state);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).unwrap(), log);
    assert_eq!(fx.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &gitio::RunOpts::default()).unwrap().stdout, all_refs);
}

#[test]
fn duplicate_zone_reports_its_member_and_changes_nothing() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "alpha", ID_B, "b1", "b\n");
    let ia = landed(&fx, &a);
    let before = fx.landing_json();
    let log = std::fs::read(oplog_path(&fx.common)).unwrap();
    assert_eq!(land_batch(&fx.git, &refs(), &b, ACTOR).unwrap(), BatchLandAnswer::ZoneInBatch { zone: "alpha".into(), change_id: ChangeId::parse(ID_A).unwrap() });
    assert_eq!(land(&fx.git, &refs(), &b, ACTOR).unwrap(), LandAnswer::Busy { change_id: ChangeId::parse(ID_A).unwrap() });
    assert_eq!(fx.branch("integration"), ia);
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.landing_json(), before);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).unwrap(), log);
}

#[test]
fn off_tip_is_busy_with_newest_member() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    let c = squash(&fx, "gamma", ID_C, "c1", "c\n");
    seed(&fx, &[(ID_A, "alpha", &a, &a.commit), (ID_B, "beta", &b, &b.commit)]);
    fx.set_branch("integration", &c.commit);
    let before = fx.landing_json();
    let log = std::fs::read(oplog_path(&fx.common)).ok();
    assert_eq!(land_batch(&fx.git, &refs(), &c, ACTOR).unwrap(), BatchLandAnswer::Busy { change_id: ChangeId::parse(ID_B).unwrap() });
    assert_eq!(fx.branch("integration"), c.commit);
    assert_eq!(fx.landing_json(), before);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).ok(), log);
}

#[test]
fn repeat_earlier_member_returns_its_own_commit() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    seed(&fx, &[(ID_A, "alpha", &a, &a.commit), (ID_B, "beta", &b, &b.commit)]);
    let before = fx.landing_json();
    let log = std::fs::read(oplog_path(&fx.common)).ok();
    assert_eq!(land_batch(&fx.git, &refs(), &a, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: a.commit.clone() });
    assert_eq!(fx.branch("integration"), b.commit);
    assert_eq!(fx.landing_json(), before);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).ok(), log);
}

#[test]
fn back_out_restores_origin_for_every_member() {
    let fx = Fixture::new();
    let a = squash(&fx, "alpha", ID_A, "a1", "a\n");
    let b = squash(&fx, "beta", ID_B, "b1", "b\n");
    let c = squash(&fx, "gamma", ID_C, "c1", "c\n");
    seed(&fx, &[(ID_A, "alpha", &a, &a.commit), (ID_B, "beta", &b, &b.commit), (ID_C, "gamma", &c, &c.commit)]);
    assert_eq!(back_out_batch(&fx.git, &refs(), ACTOR).unwrap(), BatchBackOutAnswer::BackedOut {
        change_id: ChangeId::parse(ID_C).unwrap(), change_ids: [ID_A, ID_B, ID_C].map(|id| ChangeId::parse(id).unwrap()).to_vec(),
    });
    assert_eq!(fx.branch("integration"), fx.base);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(read_op_log(&fx.common).unwrap().records.iter().filter(|r| r.kind == "land-back-out" && r.phase == OpPhase::Done).count(), 1);
}
