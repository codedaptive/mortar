//! Independent real Git object fixtures; only the fixture's disposable files/refs are changed.
use crate::support::{oid, raw, Fixture, REQUEST, ZONE};
use gitio::{Oid, RunOpts};

pub fn tree(f: &Fixture, parent: &Oid, blobs: &[(&str, &[u8])], remove: &[&str], links: &[(&str, &Oid)]) -> Oid {
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), f.common.join("resolve-rit-oracle-index").into_os_string())], ..RunOpts::default() };
    f.git.run(&["read-tree", parent.as_str()], &opts).unwrap();
    for path in remove { f.git.run(&["update-index", "--force-remove", "--", path], &opts).unwrap(); }
    for (path, bytes) in blobs {
        let id = oid(&f.git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap().stdout);
        f.git.run(&["update-index", "--add", "--cacheinfo", "100644", id.as_str(), path], &opts).unwrap();
    }
    for (path, id) in links { f.git.run(&["update-index", "--add", "--cacheinfo", "160000", id.as_str(), path], &opts).unwrap(); }
    oid(&f.git.run(&["write-tree"], &opts).unwrap().stdout)
}

pub fn commit(f: &Fixture, parent: &Oid, tree: &Oid, message: &str) -> Oid {
    f.git.commit_tree(tree, std::slice::from_ref(parent), message, None, None).unwrap()
}

pub fn set_zone(f: &Fixture, tip: &Oid) { raw(&f.git, &["update-ref", ZONE, tip.as_str()]); }

/// Called only while constructing a fresh fixture, before any local work is added.
pub fn set_request(f: &Fixture, tip: &Oid) {
    raw(&f.git, &["update-ref", REQUEST, tip.as_str()]);
    raw(&f.local(), &["checkout", "--force", "req/core/task"]);
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{}\n", f.git.commit_info(tip).unwrap().tree).as_bytes());
}

pub fn rebase_records(f: &Fixture) -> Vec<oplog::OpRecord> {
    f.records().into_iter().filter(|r| r.kind == "join-rebase").collect()
}

pub fn assert_one_rebase(f: &Fixture, old: &Oid, new: &Oid) {
    let records = rebase_records(f);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].phase, oplog::OpPhase::Intent);
    assert_eq!(records[1].phase, oplog::OpPhase::Done);
    assert_eq!(records[1].result, Some(oplog::OpResult::Moved));
    assert_eq!(records[0].op_id, records[1].op_id);
    for r in records {
        assert_eq!(r.actor, "core/task");
        assert_eq!(r.moves, vec![oplog::RefMove { refname: REQUEST.into(), old: Some(old.clone()), new: new.clone() }]);
    }
}
