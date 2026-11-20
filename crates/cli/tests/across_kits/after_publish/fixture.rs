//! Owned real CLI/Folders/Join/Land/Oplog setup. Reuses the existing isolated Git/process fixture, never a team repo.
use crate::{json::{check, Report}, bisect_backup::fixture::Harness, support::{raw, write}};
use gitio::{Git, Oid};
use land::{LandAnswer, PublishAnswer, Squash, ZoneRequests};
use std::{fs, path::{Path, PathBuf}};

pub struct World { pub s: Harness, pub b: PathBuf }
impl World {
    pub fn new() -> Self {
        let s = Harness::new();
        let mut config = s.f.init_args.clone();
        config.zones.insert("b".into(), crate::support::ZoneConfig { paths: vec!["src".into(), "sub".into()] });
        s.f.configure(&config);
        let b = s.f.root.join("zone-b");
        s.app(&["folder", "zone", "b", b.to_str().unwrap(), "--start", s.f.base.as_str()], 0, "made");
        Self { s, b }
    }
    pub fn git(&self, folder: &Path) -> Git { self.s.git_at(folder) }
    pub fn tip(&self, zone: &str) -> Oid { self.s.f.git.read_ref(&format!("refs/heads/zone/{zone}")).unwrap().unwrap() }
    pub fn request(&self, zone: &str, name: &str) -> PathBuf {
        let path = self.s.f.root.join(format!("request-{zone}-{name}"));
        self.s.app(&["folder", "request", zone, name, path.to_str().unwrap(), "--paths", "src", "sub"], 0, "made");
        path
    }
    pub fn join(&self, folder: &Path, zone: &str, name: &str, edits: &[(&str, &[u8])]) -> Oid {
        for (path, bytes) in edits { write(folder, path, bytes); }
        check(self.s.f.command(folder, &["--json", "join"]).output().unwrap(), "join", 0, "joined");
        let tip = self.s.f.git.read_ref(&format!("refs/heads/req/{zone}/{name}")).unwrap().unwrap();
        assert_eq!(self.tip(zone), tip); tip
    }
    pub fn core(&self, edits: &[(&str, &[u8])]) -> Oid { self.join(&self.s.f.request, "core", "task", edits) }
    pub fn b_join(&self, name: &str, edits: &[(&str, &[u8])]) -> Oid {
        let path = self.request("b", name); self.join(&path, "b", name, edits)
    }
    pub fn publish_squash(&self, squash: &Squash) -> Oid {
        self.s.locked(|| {
            let LandAnswer::Landed { integration } = land::land(&self.s.f.git, &self.s.refs(), squash, "after-publish-setup").unwrap() else { panic!("not landed") };
            assert_eq!(land::publish(&self.s.f.git, &self.s.refs(), "after-publish-setup").unwrap(), PublishAnswer::Published { origin: integration.clone() });
            integration
        })
    }
    pub fn publish(&self) -> (Squash, Oid) {
        let squash = self.s.squash(&self.tip("core")); let origin = self.publish_squash(&squash); (squash, origin)
    }
    pub fn after(&self, exit: i32, outcome: &str) -> Report {
        let r = self.s.app(&["after-publish", "core", "--move", "b"], exit, outcome);
        assert!(!self.s.f.common.join("mortar/locks/land.lock").exists()); r
    }
    /// Land-only completion intentionally leaves the actual zone folders behind for CLI catch-up cases.
    pub fn kit_after(&self) {
        let zone = |name: &str| ZoneRequests { zone: name.into(), requests: vec![] };
        self.s.locked(|| land::after_publish(&self.s.f.git, &self.s.refs(), &zone("core"), &[zone("b")], "after-publish-setup").unwrap());
    }
    pub fn folder_at(&self, folder: &Path, tip: &Oid, files: &[(&str, &[u8])]) {
        assert_eq!(self.s.f.git.index_tree(folder).unwrap(), self.s.f.git.commit_info(tip).unwrap().tree);
        for (path, bytes) in files { assert_eq!(fs::read(folder.join(path)).unwrap(), *bytes, "{path}"); }
        assert_eq!(raw(&self.git(folder), &["status", "--porcelain"]), b"");
    }
    /// All real index entries and file bytes, including untracked obstructions, independently of the moving HEAD.
    pub fn folder_state(&self, folder: &Path) -> (Vec<u8>, Vec<(PathBuf, Vec<u8>)>) {
        fn files(root: &Path, at: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in fs::read_dir(at).unwrap() {
                let path = entry.unwrap().path(); if path.file_name().unwrap() == ".git" { continue; }
                if path.is_dir() { files(root, &path, out); } else { out.push((path.strip_prefix(root).unwrap().into(), fs::read(path).unwrap())); }
            }
        }
        let mut bytes = Vec::new(); files(folder, folder, &mut bytes); bytes.sort_by(|a, b| a.0.cmp(&b.0));
        (raw(&self.git(folder), &["ls-files", "--stage", "-z"]), bytes)
    }
    pub fn store(&self) -> Vec<Vec<u8>> {
        vec![raw(&self.s.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), self.s.f.log().unwrap_or_default(),
            self.s.remote_refs(), raw(&self.s.f.git, &["worktree", "list", "--porcelain"]),
            fs::read(self.s.f.common.join("mortar/landing.json")).unwrap_or_default()]
    }
    pub fn assert_origin(&self, origin: &Oid) {
        assert_eq!(self.s.f.git.read_ref("refs/heads/main").unwrap(), Some(origin.clone()));
        assert_eq!(self.s.f.git.read_ref("refs/heads/integration").unwrap(), Some(origin.clone()));
        assert_eq!(self.s.git_at(&self.s.bare).read_ref("refs/heads/main").unwrap(), Some(origin.clone()));
    }
}
