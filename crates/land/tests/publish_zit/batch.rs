use crate::fixture::{refs, tree, Fixture, ACTOR, ID_A, SQUASH_FILES, SQUASH_REF};
use gitio::Oid;
use land::{land_batch, publish, BatchLandAnswer, PublishAnswer, Squash};
use oplog::{apply, read_op_log, ChangeId, Op, OpPhase, RefMove};

const ID_B: &str = "zyxwvutsrqponmlkzyxwvutsrqponmlk";
const REF_B: &str = "refs/mortar/squash/other";

fn second_and_tip(fx: &Fixture, first: &Oid) -> (Squash, Oid) {
    let second = Squash {
        commit: fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("b.txt", "second\n")]),
            std::slice::from_ref(&fx.base), &format!("other squash\n\nMortar-Change-Id: {ID_B}\nMortar-Zone: other\n"), None, None).unwrap(),
        change_id: ChangeId::parse(ID_B).unwrap(), source: fx.base.clone(), requests: vec!["r1".into()],
    };
    let tip = fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("b.txt", "second\n"), ("s.txt", "squash\n")]),
        &[first.clone(), second.commit.clone()], &format!("land {ID_B} (other)\n"), None, None).unwrap();
    (second, tip)
}

fn batch() -> (Fixture, Squash, Squash, Oid) {
    let fx = Fixture::new();
    let first = fx.squash(&fx.base, &SQUASH_FILES);
    let (second, tip) = second_and_tip(&fx, &first.commit);
    fx.set_ref(SQUASH_REF, &first.commit);
    fx.set_ref(REF_B, &second.commit);
    assert_eq!(land_batch(&fx.git, &refs(), &first, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: first.commit.clone() });
    assert_eq!(land_batch(&fx.git, &refs(), &second, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: tip.clone() });
    (fx, first, second, tip)
}

#[test]
fn publish_batch_pushes_once_and_deletes_both_squashes_in_order() {
    let (fx, _, _, tip) = batch();
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: tip.clone() });
    assert_eq!(fx.remote_main(), Some(tip.clone()));
    assert_eq!(fx.branch("main"), tip);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert_eq!(fx.git.read_ref(REF_B).unwrap(), None);
    assert_eq!(fx.landing_json(), None);
    let records = read_op_log(&fx.common).unwrap().records;
    let done: Vec<_> = records.iter().filter(|r| r.phase == OpPhase::Done && r.kind.starts_with("publish-")).collect();
    assert_eq!(done.iter().map(|r| r.kind.as_str()).collect::<Vec<_>>(), vec!["publish-push", "publish-origin", "publish-squash", "publish-squash"]);
    for (record, id) in done.iter().zip([ID_B, ID_B, ID_A, ID_B]) {
        assert!(record.reason.starts_with(&format!("{} {id} ", record.kind)), "{}", record.reason);
    }
    assert_eq!(done[2].moves[0].refname, SQUASH_REF);
    assert_eq!(done[3].moves[0].refname, REF_B);
}

#[test]
fn resquashed_member_is_kept_and_other_member_is_deleted() {
    let (fx, _, _, tip) = batch();
    fx.set_ref(REF_B, &fx.base);
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: tip });
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert_eq!(fx.git.read_ref(REF_B).unwrap(), Some(fx.base.clone()));
}

#[test]
fn repeat_after_first_squash_cleanup_deletes_the_second_once() {
    let (fx, first) = Fixture::landed();
    let (second, tip) = second_and_tip(&fx, &first.commit);
    fx.set_ref(REF_B, &second.commit);
    fx.set_ref("refs/heads/integration", &tip);
    let text = format!("{{\"origin_at_start\":\"{}\",\"landings\":[{{\"change_id\":\"{ID_A}\",\"zone\":\"land\",\"squash\":\"{}\",\"integration\":\"{}\"}},{{\"change_id\":\"{ID_B}\",\"zone\":\"other\",\"squash\":\"{}\",\"integration\":\"{tip}\"}}]}}\n", fx.base, first.commit, first.commit, second.commit);
    std::fs::write(fx.common.join("mortar/landing.json"), text).unwrap();
    fx.push_remote_main(&tip);
    fx.set_ref("refs/heads/main", &tip);
    let zero = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    let op = Op {
        op_id: "batch-cleanup-first".into(), key: "batch-cleanup-first".into(), kind: "publish-squash".into(),
        actor: ACTOR.into(), reason: format!("publish-squash {ID_A} first member"), at: 946_684_800,
        moves: vec![RefMove { refname: SQUASH_REF.into(), old: Some(first.commit), new: zero }],
    };
    apply(&fx.git, &fx.common, &op).unwrap();
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: tip });
    assert_eq!(fx.git.read_ref(REF_B).unwrap(), None);
    assert_eq!(fx.landing_json(), None);
    let records = read_op_log(&fx.common).unwrap().records;
    let done: Vec<_> = records.iter().filter(|r| r.kind == "publish-squash" && r.phase == OpPhase::Done).collect();
    assert_eq!(done.len(), 2);
    assert_eq!(done[0].moves[0].refname, SQUASH_REF);
    assert_eq!(done[1].moves[0].refname, REF_B);
    assert!(done[1].reason.starts_with(&format!("publish-squash {ID_B} ")));
}
