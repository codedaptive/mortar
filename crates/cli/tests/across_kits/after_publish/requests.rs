//! read_named_zones/run_after_publish joined to actual folder records, snapshots, request ancestry and Land replay.
use super::fixture::World;
use crate::support::{raw, write};
use oplog::{KillPidProbe, LockName};
use std::fs;

#[test]
fn absent_zone_folder_still_lists_open_requests_using_their_actual_snapshot_tips() {
    let w = World::new(); let old = w.b_join("r3", &[("src/r3", b"joined three\n")]);
    let plain = w.request("b", "plain"); let unsaved = w.request("b", "unsaved");
    write(&unsaved, "src/local", b"not saved on the request branch\n");
    let snapshot = oplog::with_lock(&w.s.f.git, &w.s.f.common, &LockName::Join("b".into()), "after-publish-snapshot", &KillPidProbe, || {
        oplog::recover(&w.s.f.git, &w.s.f.common).unwrap();
        snapshot::snapshot(&w.s.f.git, &unsaved, &snapshot::SnapshotTarget { zone: "b".into(), request: "unsaved".into(), attempt: None, actor: "after-publish".into() }, &[], snapshot::DEFAULT_SIZE_CAP)
    }).unwrap();
    assert!(snapshot.written); assert_ne!(snapshot.commit, old);
    assert_eq!(w.s.f.git.read_ref("refs/heads/req/b/unsaved").unwrap(), Some(old.clone()));
    assert_eq!(w.s.f.blob(&snapshot.commit, "src/local"), b"not saved on the request branch\n");
    // Join advanced the branch, not this folder: first catch up the owned, untouched folder so Git removes it cleanly.
    raw(&w.git(&w.b), &["read-tree", "-m", "-u", w.s.f.base.as_str(), old.as_str()]);
    w.folder_at(&w.b, &old, &[("src/r3", b"joined three\n")]);
    raw(&w.s.f.git, &["worktree", "remove", w.b.to_str().unwrap()]); assert!(!w.b.exists());
    let before_read = w.store();
    assert_eq!(raw(&w.s.f.git, &["for-each-ref", "--format=%(refname)", "refs/heads/req/b/"]),
        b"refs/heads/req/b/plain\nrefs/heads/req/b/r3\nrefs/heads/req/b/unsaved\n");
    assert_eq!(w.s.f.git.read_ref("refs/heads/req/b/plain").unwrap(), Some(old.clone()));
    assert_eq!(w.s.f.git.read_ref("refs/mortar/snap/b/unsaved").unwrap(), Some(snapshot.commit.clone()));
    assert_eq!(w.s.f.git.commit_info(&snapshot.commit).unwrap().parents.first(), Some(&old));
    let read = w.s.app(&["overlap", "b"], 0, "no-overlap");
    assert!(read.strings("pairs").is_empty()); assert_eq!(w.store(), before_read);
    w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish();
    let request_states = (w.folder_state(&plain), w.folder_state(&unsaved));
    let report = w.after(0, "after-published");
    assert_eq!(report.strings("moved"), &["core", "b"]);
    assert!(report.strings("lines").contains(&"zone b has no zone folder".into()));
    for name in ["r3", "plain", "unsaved"] {
        assert!(report.strings("rebase").contains(&name.into()));
        assert!(report.strings("lines").contains(&format!("request {name} rebases at its next join")));
    }
    assert_eq!(report.text("next"), "run mr join in the folder of each request listed");
    assert_eq!(report.strings("folders_moved"), &[w.s.f.zone.display().to_string()]);
    assert_eq!(w.s.f.git.read_ref("refs/mortar/snap/b/unsaved").unwrap(), Some(snapshot.commit));
    assert_eq!((w.folder_state(&plain), w.folder_state(&unsaved)), request_states); w.assert_origin(&origin);
}

#[test]
fn unrelated_request_missing_zone_and_unreadable_folder_record_stop_before_land_or_folder_movement() {
    for kind in ["unrelated", "missing-zone", "bad-record"] {
        let w = World::new(); w.core(&[("src/file", b"published\n")]); let (_, origin) = w.publish();
        let request = w.request("b", "victim"); let git_dir = w.s.f.git.git_dir(&request).unwrap();
        let record_path = folders::record::record_path(&git_dir);
        let expected = match kind {
            "unrelated" => {
                let tree = w.s.f.git.commit_info(&w.s.f.base).unwrap().tree;
                let root = w.s.f.git.commit_tree(&tree, &[], "a different root\n", None, None).unwrap();
                raw(&w.s.f.git, &["update-ref", "refs/heads/req/b/victim", root.as_str()]);
                "the request victim shares no commit with zone b".to_string()
            }
            "missing-zone" => {
                raw(&w.s.f.git, &["update-ref", "-d", "refs/heads/zone/b"]);
                "refs/heads/zone/b does not exist".to_string()
            }
            "bad-record" => {
                fs::write(&record_path, b"not a Mortar folder record\n").unwrap();
                format!("the folder record of {} cannot be read", request.display())
            }
            _ => unreachable!(),
        };
        let before = w.store(); let folders = (w.folder_state(&w.s.f.zone), w.folder_state(&w.b), w.folder_state(&request));
        let record = fs::read(&record_path).unwrap(); let r = w.after(5, "failed");
        assert_eq!(r.strings("lines")[0], expected);
        assert_eq!(w.store(), before); assert_eq!(fs::read(record_path).unwrap(), record);
        assert_eq!((w.folder_state(&w.s.f.zone), w.folder_state(&w.b), w.folder_state(&request)), folders);
        assert_eq!(fs::read(w.s.f.zone.join("src/file")).unwrap(), b"base\n"); w.assert_origin(&origin);
    }
}
