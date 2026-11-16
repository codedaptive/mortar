//! WO-land-rit-4 (S-109, S-119, S-121, S-129): public land::after_publish -> oplog::{undo, restore}.
//! A kept squash consumes the replay record an earlier publish wrote; undo restores both of its op's refs,
//! and restore to the earlier replay op recreates the record and zone tip. Local and remote origin stay published.
//! Existing support owns/removes the disposable repository and its nested bare remote; no private Land API is used.
use crate::{owed_ops::{locked, raw, refs}, support::Fixture};
use gitio::{Git, Oid, RunOpts};
use land::{LandAnswer, PublishAnswer, Squash, SquashAnswer, ZoneMove, ZoneRequests};
use oplog::{ApplyOutcome, OpCaller, OpPhase, OpRecord, RefMove};

const ACTOR: &str = "land-rit-4";
const ZONE: &str = "refs/heads/zone/core";
const MAIN: &str = "refs/heads/main";
const ZERO: &str = "0000000000000000000000000000000000000000";

struct Consumed {
    f: Fixture,
    remote: Git,
    origin: Oid,
    replayed: Oid,
    record: String,
    replay_op: String,
}

fn output_id(f: &Fixture, args: &[&str], input: &[u8]) -> Oid {
    let out = f.git.run(args, &RunOpts { stdin: Some(input.to_vec()), ..RunOpts::default() }).unwrap();
    Oid::parse(std::str::from_utf8(&out.stdout).unwrap().trim_end()).unwrap()
}

fn made(f: &Fixture, zone: &str) -> Squash {
    let answer = locked(f, || land::squash(&f.git, &refs(), zone, ACTOR));
    let SquashAnswer::Made(squash) = answer else { panic!("fixture must squash {zone}: {answer:?}") };
    squash
}

fn published(f: &Fixture, squash: &Squash) -> Oid {
    let answer = locked(f, || land::land(&f.git, &refs(), squash, ACTOR));
    let LandAnswer::Landed { integration } = answer else { panic!("fixture must land: {answer:?}") };
    assert_eq!(locked(f, || land::publish(&f.git, &refs(), ACTOR)), PublishAnswer::Published { origin: integration.clone() });
    integration
}

fn done_after_publish(f: &Fixture) -> Vec<OpRecord> {
    oplog::read_op_log(&f.common).unwrap().records.into_iter()
        .filter(|r| r.kind == "after-publish" && r.phase == OpPhase::Done).collect()
}

fn zone(name: &str) -> ZoneRequests { ZoneRequests { zone: name.into(), requests: vec![] } }

impl Consumed {
    fn new() -> Self {
        let f = Fixture::new(); let base = f.git.read_ref(MAIN).unwrap().unwrap();
        let remote_path = f.git.repo.join("rit4-origin.git");
        raw(&f, &["init", "--bare", "--quiet", "--object-format=sha1", remote_path.to_str().unwrap()]);
        raw(&f, &["remote", "add", "origin", remote_path.to_str().unwrap()]);
        raw(&f, &["push", "--quiet", "origin", &format!("{base}:{MAIN}")]);
        let remote = Git { repo: remote_path, ..f.git.clone() };
        // Core changes `file`; b keeps the base's tree and adds only `peer`, so their replay is independent.
        let mut listing = f.git.run(&["ls-tree", base.as_str()], &RunOpts::default()).unwrap().stdout;
        let blob = output_id(&f, &["hash-object", "-w", "--stdin"], b"peer work\n");
        listing.extend_from_slice(format!("100644 blob {blob}\tpeer\n").as_bytes());
        let tree = output_id(&f, &["mktree"], &listing);
        let b = f.git.commit_tree(&tree, &[base], "peer work\n", None, None).unwrap();
        raw(&f, &["update-ref", "refs/heads/zone/b", b.as_str()]);
        let kept = made(&f, "core");
        let b_origin = published(&f, &made(&f, "b"));
        let moved = locked(&f, || land::after_publish(&f.git, &refs(), &zone("b"), &[zone("core")], ACTOR));
        let [ZoneMove::Moved { old: b_old, new: b_new, .. }, ZoneMove::Moved { zone: name, old, new, rebase }] = moved.as_slice() else {
            panic!("b advances and core replays: {moved:?}");
        };
        assert_eq!((b_old, b_new), (&b, &b_origin));
        assert_eq!((name.as_str(), old, rebase), ("core", &f.source, &vec![]));
        let replayed = new.clone();
        assert_eq!(f.git.commit_info(&replayed).unwrap().parents, vec![b_origin.clone()]);
        for (path, bytes) in [("file", b"zone change\n".as_slice()), ("peer", b"peer work\n".as_slice())] {
            let out = f.git.run(&["cat-file", "blob", &format!("{replayed}:{path}")], &RunOpts::default()).unwrap();
            assert_eq!(out.stdout, bytes);
        }
        let record = format!("refs/mortar/replayed/core/{}", f.source);
        assert_eq!(f.git.read_ref(&record).unwrap(), Some(replayed.clone()));
        let replay_op = done_after_publish(&f).last().unwrap().op_id.clone();
        let origin = published(&f, &kept);
        assert_eq!(f.git.commit_info(&origin).unwrap().parents, vec![b_origin, kept.commit]);
        let before = done_after_publish(&f);
        assert_eq!(locked(&f, || land::after_publish(&f.git, &refs(), &zone("core"), &[], ACTOR)),
            vec![ZoneMove::Moved { zone: "core".into(), old: replayed.clone(), new: origin.clone(), rebase: vec![] }]);
        let after = done_after_publish(&f); assert_eq!(after.len(), before.len() + 1);
        assert_eq!(after.last().unwrap().moves, vec![
            RefMove { refname: ZONE.into(), old: Some(replayed.clone()), new: origin.clone() },
            RefMove { refname: record.clone(), old: Some(replayed.clone()), new: Oid::parse(ZERO).unwrap() },
        ]);
        assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(origin.clone()));
        assert_eq!(f.git.read_ref(&record).unwrap(), None);
        let consumed = Self { f, remote, origin, replayed, record, replay_op };
        consumed.origin_unchanged(); consumed
    }

    fn origin_unchanged(&self) {
        assert_eq!(self.f.git.read_ref(MAIN).unwrap(), Some(self.origin.clone()));
        assert_eq!(self.f.git.read_ref("refs/heads/integration").unwrap(), Some(self.origin.clone()));
        assert_eq!(self.remote.read_ref(MAIN).unwrap(), Some(self.origin.clone()));
    }

    fn restored(&self, outcome: ApplyOutcome, kind: &str) {
        let ApplyOutcome::Done(done) = outcome else { panic!("fresh {kind} must run: {outcome:?}") };
        assert_eq!(done.kind, kind); assert_eq!(done.phase, OpPhase::Done);
        assert_eq!(done.moves, vec![
            RefMove { refname: ZONE.into(), old: Some(self.origin.clone()), new: self.replayed.clone() },
            RefMove { refname: self.record.clone(), old: None, new: self.replayed.clone() },
        ]);
        assert_eq!(self.f.git.read_ref(ZONE).unwrap(), Some(self.replayed.clone()));
        assert_eq!(self.f.git.read_ref(&self.record).unwrap(), Some(self.replayed.clone()));
        self.origin_unchanged();
    }
}

fn caller(kind: &str) -> OpCaller {
    OpCaller { key: format!("rit4-{kind}-consumed-record"), actor: ACTOR.into(), reason: "recover consumed replay record".into() }
}

#[test]
fn rit_after_publish_consumed_record_and_zone_are_restored_by_oplog_undo() {
    let p = Consumed::new();
    // Scope only the zone: undo must reverse the entire Land op, including its hidden record delete.
    let answer = locked(&p.f, || oplog::undo(&p.f.git, &p.f.common, &[ZONE.into()], &caller("undo"), &[]));
    p.restored(answer, "undo");
}

#[test]
fn rit_after_publish_consumed_record_and_zone_are_restored_to_the_replay_op() {
    let p = Consumed::new();
    let answer = locked(&p.f, || oplog::restore(&p.f.git, &p.f.common, &p.replay_op,
        &[ZONE.into(), p.record.clone()], &caller("restore"), &[]));
    p.restored(answer, "restore");
}
