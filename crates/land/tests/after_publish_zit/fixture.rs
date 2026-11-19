//! The after_publish ZITs' throwaway repositories: a local repository with a base commit on `main` and
//! `integration`, and a bare remote `origin` (inside a second TestRepo's directory, removed with it) whose `main` is
//! the base. Zone commits are made with plain git; the publish itself is land's own squash, land and publish, so
//! after_publish reads the trailers squash wrote. made, land_and_publish and origin_untouched are the cases' shared
//! steps for a kept squash published later; replayed, replayed_chain and superseded are the shared arrangements
//! (zone `land` replayed onto b's origin at A1, or at A2 on A1, and an older squash landed after a newer one
//! published); record_ref, deleted, zone_move and done_moves read and spell the op log's replay record moves.

use gitio::{Git, Oid, RunOpts};
use join::OpenRequest;
use land::{
    after_publish, land, publish, squash, LandAnswer, LandRefs, PublishAnswer, Squash, SquashAnswer, ZoneMove,
    ZoneRequests,
};
use oplog::{read_op_log, OpPhase, OpRecord, RefMove};
use std::ffi::OsString;
use std::path::PathBuf;
use testkit::TestRepo;

/// The zone whose squash every case publishes.
pub const PUBLISHED: &str = "land";
/// The actor every case's own squash, land, publish and after_publish record.
pub const ACTOR: &str = "zit@host";
/// The base commit's files: a.txt holds three lines the zones edit.
pub const BASE_FILES: [(&str, &str); 1] = [("a.txt", "one\ntwo\nthree\n")];
/// The published zone's one commit on the base: a.txt's middle line changed, p.txt added.
pub const PUBLISHED_FILES: [(&str, &str); 2] = [("a.txt", "one\nTWO\nthree\n"), ("p.txt", "published\n")];
/// Zone b's commit on the base: b.txt added, a.txt untouched.
pub const B_FILES: [(&str, &str); 2] = [("a.txt", "one\ntwo\nthree\n"), ("b.txt", "b\n")];
/// Zone c's commit on the base: the same middle line of a.txt the published zone changed, changed otherwise.
pub const C_FILES: [(&str, &str); 1] = [("a.txt", "one\ntwo-c\nthree\n")];
/// A2 on A1 (PUBLISHED_FILES): A1's files plus q.txt.
pub const A2_FILES: [(&str, &str); 3] =
    [("a.txt", "one\nTWO\nthree\n"), ("p.txt", "published\n"), ("q.txt", "second\n")];
/// A join on the replayed A1': the files of A1' (b.txt carried in by the replay) plus j.txt.
pub const JOINED_FILES: [(&str, &str); 4] =
    [("a.txt", "one\nTWO\nthree\n"), ("b.txt", "b\n"), ("j.txt", "joined\n"), ("p.txt", "published\n")];
/// The all-zero id the op log writes as a deleted ref's new value (the fixture's repositories are SHA-1).
const ZERO: &str = "0000000000000000000000000000000000000000";

pub struct Fixture {
    /// The local repository's directory, removed when the fixture drops.
    _repo: TestRepo,
    pub git: Git,
    pub common: PathBuf,
    /// The first commit, where local `main` and `integration` and the remote's `main` start.
    pub base: Oid,
    /// Holds the bare remote; removed when the fixture drops.
    _elsewhere: TestRepo,
    /// The bare remote's git.
    pub remote: Git,
}

/// Git's config isolated and a fixed identity and dates, so commit_tree needs no user config.
fn git_env() -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_AUTHOR_NAME", "Zit"),
        ("GIT_AUTHOR_EMAIL", "zit@example.invalid"),
        ("GIT_AUTHOR_DATE", "2000-01-01T00:00:00+00:00"),
        ("GIT_COMMITTER_NAME", "Zit"),
        ("GIT_COMMITTER_EMAIL", "zit@example.invalid"),
        ("GIT_COMMITTER_DATE", "2000-01-01T00:00:00+00:00"),
    ]
    .map(|(key, value)| (key.into(), value.into()))
    .to_vec()
}

pub fn refs() -> LandRefs {
    LandRefs {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
    }
}

/// A zone with the open requests the caller hands in.
pub fn zone(name: &str, requests: Vec<OpenRequest>) -> ZoneRequests {
    ZoneRequests { zone: name.to_string(), requests }
}

/// An open request cut from `base`, its tip `tip`.
pub fn request(name: &str, base: &Oid, tip: &Oid) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: base.clone(), tip: tip.clone() }
}

/// The zone's squash, made with land's own squash.
pub fn made(fx: &Fixture, zone: &str) -> Squash {
    let SquashAnswer::Made(made) = squash(&fx.git, &refs(), zone, ACTOR).unwrap() else {
        panic!("zone {zone} must squash");
    };
    made
}

/// The squash landed and published with land's own code; the origin publish answers.
pub fn land_and_publish(fx: &Fixture, made: &Squash) -> Oid {
    let LandAnswer::Landed { integration } = land(&fx.git, &refs(), made, ACTOR).unwrap() else {
        panic!("the squash must land");
    };
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: integration.clone() });
    integration
}

/// main and the remote's main are both `origin`.
pub fn origin_untouched(fx: &Fixture, origin: &Oid) {
    assert_eq!(fx.branch("main"), *origin);
    assert_eq!(fx.remote_main(), Some(origin.clone()));
}

/// The replay record of `commit` in zone `zone`.
pub fn record_ref(zone: &str, commit: &Oid) -> String {
    format!("refs/mortar/replayed/{zone}/{}", commit.as_str())
}

/// The op log's delete of `refname`, which held `old`.
pub fn deleted(refname: &str, old: &Oid) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(old.clone()), new: Oid::parse(ZERO).unwrap() }
}

/// The op log's move of `refname` from `old` to `new`, alone.
pub fn zone_move(refname: &str, old: &Oid, new: &Oid) -> Vec<RefMove> {
    vec![RefMove { refname: refname.to_string(), old: Some(old.clone()), new: new.clone() }]
}

/// The moves of the log's done after-publish records, in log order.
pub fn done_moves(fx: &Fixture) -> Vec<Vec<RefMove>> {
    let records = fx.after_publish_records().into_iter().filter(|record| record.phase == OpPhase::Done);
    records.map(|record| record.moves).collect()
}

/// Zone `land` at A1 (PUBLISHED_FILES on the base) with its squash kept, zone `b` (B_FILES on the base) squashed,
/// landed and published, then after_publish(b, others=[land]).
pub struct Replayed {
    pub a1: Oid,
    /// Land's kept squash, its Mortar-Squash-Of A1.
    pub land_squash: Squash,
    pub b_origin: Oid,
    /// The replay of A1 onto b's origin: zone `land`'s tip.
    pub a1_replayed: Oid,
}

/// The kept-squash arrangement up to the replay, checked to move zone `land` from A1 to a commit on b's origin with
/// no open request listed.
pub fn replayed(fx: &Fixture) -> Replayed {
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let land_squash = made(fx, PUBLISHED);
    let b_origin = land_and_publish(fx, &made(fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    let ZoneMove::Moved { zone: name, old, new: a1_replayed, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto b's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &a1, &vec![]));
    assert_eq!(fx.git.commit_info(a1_replayed).unwrap().parents, vec![b_origin.clone()]);
    Replayed { a1_replayed: a1_replayed.clone(), a1, land_squash, b_origin }
}

/// Zone `land` at A2 on A1, its squashes already made, replayed onto b's origin by after_publish(b, others=[land]).
pub struct ChainReplayed {
    pub b_origin: Oid,
    pub a1_replayed: Oid,
    pub a2_replayed: Oid,
}

/// Zone `b` (B_FILES on the base) squashed, landed and published, then after_publish(b, others=[land]) with zone
/// `land` at `a2` on `a1`: checked to put b on its origin and to replay A1 and A2 one for one onto it, each recorded
/// at its replay record.
pub fn replayed_chain(fx: &Fixture, a1: &Oid, a2: &Oid) -> ChainReplayed {
    let b_tip = fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let b_origin = land_and_publish(fx, &made(fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    assert_eq!(moves[0], ZoneMove::Moved { zone: "b".to_string(), old: b_tip, new: b_origin.clone(), rebase: vec![] });
    let ZoneMove::Moved { zone: name, old, new: a2_replayed, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto b's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, a2, &vec![]));
    assert_eq!(fx.files(a2_replayed), ["a.txt", "b.txt", "p.txt", "q.txt"]);
    let a1_replayed = fx.git.commit_info(a2_replayed).unwrap().parents[0].clone();
    assert_eq!(fx.git.commit_info(&a1_replayed).unwrap().parents, vec![b_origin.clone()]);
    assert_eq!(fx.files(&a1_replayed), ["a.txt", "b.txt", "p.txt"]);
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, a1)).unwrap(), Some(a1_replayed.clone()));
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, a2)).unwrap(), Some(a2_replayed.clone()));
    ChainReplayed { b_origin, a1_replayed, a2_replayed: a2_replayed.clone() }
}

/// An older squash S1 landed after a newer S2 of the same zone published.
pub struct Superseded {
    pub a1: Oid,
    pub a2: Oid,
    /// S2's publish, where after_publish(land) put zone `land`.
    pub s2_origin: Oid,
    /// S1's publish: the merge [S2's origin, S1].
    pub origin: Oid,
}

/// S1 made at A1 (PUBLISHED_FILES on the base) and kept; A2 (A2_FILES) on A1; S2 made at A2, replacing the squash
/// ref; S2 landed and published and after_publish(land) checked to put the zone on S2's origin; then S1 landed, as
/// a merge, and published. after_publish(land) for S1's publish is the caller's.
pub fn superseded(fx: &Fixture) -> Superseded {
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let s1 = made(fx, PUBLISHED);
    let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "second work");
    let s2_origin = land_and_publish(fx, &made(fx, PUBLISHED));
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR).unwrap();
    let advanced =
        ZoneMove::Moved { zone: PUBLISHED.to_string(), old: a2.clone(), new: s2_origin.clone(), rebase: vec![] };
    assert_eq!(moves, vec![advanced]);
    let origin = land_and_publish(fx, &s1);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![s2_origin.clone(), s1.commit.clone()]);
    Superseded { a1, a2, s2_origin, origin }
}

fn git_at(path: PathBuf) -> Git {
    let mut git = Git::new(path);
    git.env = git_env();
    git
}

fn run(git: &Git, args: &[&str]) -> Vec<u8> {
    git.run(args, &RunOpts::default()).unwrap().stdout
}

fn oid_out(git: &Git, args: &[&str], stdin: Vec<u8>) -> Oid {
    let output = git.run(args, &RunOpts { stdin: Some(stdin), ..RunOpts::default() }).unwrap();
    Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
}

/// A tree holding exactly `files` (name, content), each at the top level.
fn tree(git: &Git, files: &[(&str, &str)]) -> Oid {
    let mut listing = String::new();
    for (name, content) in files {
        let blob = oid_out(git, &["hash-object", "-w", "--stdin"], content.as_bytes().to_vec());
        listing.push_str(&format!("100644 blob {blob}\t{name}\n"));
    }
    oid_out(git, &["mktree"], listing.into_bytes())
}

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let git = git_at(repo.path.clone());
        let base = git.commit_tree(&tree(&git, &BASE_FILES), &[], "base\n", None, None).unwrap();
        let elsewhere = TestRepo::new().unwrap();
        let bare = elsewhere.path.join("remote.git");
        run(&git_at(elsewhere.path.clone()), &["init", "--bare", "--object-format=sha1", bare.to_str().unwrap()]);
        let remote = git_at(bare.clone());
        let fixture = Fixture { common: repo.path.join(".git"), _repo: repo, git, base, _elsewhere: elsewhere, remote };
        fixture.set_ref("refs/heads/main", &fixture.base);
        fixture.set_ref("refs/heads/integration", &fixture.base);
        run(&fixture.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&fixture.git, &["push", "origin", "refs/heads/main:refs/heads/main"]);
        fixture
    }

    pub fn set_ref(&self, refname: &str, commit: &Oid) {
        run(&self.git, &["update-ref", refname, commit.as_str()]);
    }

    /// A commit on `parent` whose tree holds exactly `files`, its message `subject`; the zone branch `zone` set to it.
    pub fn join_onto(&self, zone: &str, parent: &Oid, files: &[(&str, &str)], subject: &str) -> Oid {
        let message = format!("{subject}\n");
        let commit = self.git.commit_tree(&tree(&self.git, files), std::slice::from_ref(parent), &message, None, None).unwrap();
        self.set_ref(&format!("refs/heads/zone/{zone}"), &commit);
        commit
    }

    /// The tip of `refs/heads/<name>`.
    pub fn branch(&self, name: &str) -> Oid {
        self.git.read_ref(&format!("refs/heads/{name}")).unwrap().unwrap()
    }

    /// The remote's `main`, read in the bare repository itself.
    pub fn remote_main(&self) -> Option<Oid> {
        self.remote.read_ref("refs/heads/main").unwrap()
    }

    /// The published zone squashed, landed and published with land's own code; the published origin commit.
    pub fn publish(&self) -> Oid {
        let SquashAnswer::Made(made) = squash(&self.git, &refs(), PUBLISHED, ACTOR).unwrap() else {
            panic!("the published zone must squash");
        };
        let landed = land(&self.git, &refs(), &made, ACTOR).unwrap();
        assert_eq!(landed, LandAnswer::Landed { integration: made.commit.clone() });
        let published = publish(&self.git, &refs(), ACTOR).unwrap();
        assert_eq!(published, PublishAnswer::Published { origin: made.commit.clone() });
        made.commit
    }

    /// The content of `name` in `commit`'s tree.
    pub fn file(&self, commit: &Oid, name: &str) -> String {
        String::from_utf8(run(&self.git, &["cat-file", "blob", &format!("{}:{name}", commit.as_str())])).unwrap()
    }

    /// The names of the files in `commit`'s tree, in git's order.
    pub fn files(&self, commit: &Oid) -> Vec<String> {
        let listing = String::from_utf8(run(&self.git, &["ls-tree", "--name-only", commit.as_str()])).unwrap();
        listing.lines().map(str::to_string).collect()
    }

    /// The op log's after-publish records, in log order.
    pub fn after_publish_records(&self) -> Vec<OpRecord> {
        let records = read_op_log(&self.common).unwrap().records;
        records.into_iter().filter(|record| record.kind == "after-publish").collect()
    }
}
