//! A real paused binary join, plus the independently requested Git merge-tree version of its first conflict.
use crate::{json::Report, objects::set_zone, support::{Fixture, REQUEST, ZONE}};
use gitio::{MergeTreeOutcome, Oid};
use std::{fs, path::PathBuf};

pub struct Paused { pub f: Fixture, pub old: Oid, pub zone: Oid, pub bad: Oid, pub state: PathBuf, pub tree: Oid }
impl Paused {
    pub fn new(zone_changes_keep: bool) -> Self {
        let f = Fixture::new(); let old = f.save_topic(); let base = f.base.clone();
        let mut edits: Vec<(&str, &[u8])> = vec![("src/file", b"zone work\n"), ("src/upstream", b"advanced zone work\n")];
        if zone_changes_keep { edits.push(("sub/keep", b"zone changed this path\n")); }
        let zone = f.commit(&base, &edits, "zone advanced\n"); set_zone(&f, &zone);
        Self::start(f, &base, &old)
    }
    pub fn start(f: Fixture, base: &Oid, bad: &Oid) -> Self {
        let old = f.head(); let zone = f.git.read_ref(ZONE).unwrap().unwrap();
        let MergeTreeOutcome::Conflicted { tree, files } = f.git.merge_tree(&zone, Some(base), &zone, bad).unwrap() else {
            panic!("fixture must have a real conflict");
        };
        let report = f.report(&["--json", "join"], "join", 1, "conflicted");
        assert_eq!(report.text("commit"), bad.as_str());
        assert_eq!(report.strings("files"), files.iter().map(|p| p.to_str().unwrap().to_string()).collect::<Vec<_>>());
        let state = f.local().git_dir(&f.request).unwrap().join("mortar-join.json");
        assert_eq!(report.text("state"), state.to_str().unwrap()); assert!(state.is_file());
        let this = Self { f, old, zone, bad: bad.clone(), state, tree };
        this.assert_unmoved(); this
    }
    pub fn first(&self) -> Report { self.f.report(&["--json", "resolve"], "resolve", 1, "files-written") }
    pub fn state_bytes(&self) -> Vec<u8> { fs::read(&self.state).unwrap() }
    pub fn assert_unmoved(&self) {
        assert_eq!(self.f.head(), self.old);
        assert_eq!(self.f.git.read_ref(ZONE).unwrap(), Some(self.zone.clone()));
        assert_eq!(self.f.local().rev_parse("HEAD").unwrap(), Some(self.old.clone()));
        assert!(!self.f.lock_path().exists());
        assert_eq!(self.f.git.read_ref(REQUEST).unwrap(), Some(self.old.clone()));
    }
}
