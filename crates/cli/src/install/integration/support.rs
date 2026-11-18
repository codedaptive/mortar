//! Isolated filesystem fixtures for install's integration tests.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::{install_in, InstallEnv, InstallOptions, SkillLocation};
use crate::report::Report;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub(super) struct Tree {
    pub root: PathBuf,
    pub env: InstallEnv,
}

impl Tree {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mortar-install-zit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let home = root.join("home");
        let project = root.join("project");
        let source = root.join("source");
        let prefix = root.join("prefix");
        for path in [&home, &project, &source, &prefix] {
            fs::create_dir(path).unwrap();
        }
        let exe = source.join("mortar");
        let mut bytes = b"fake mortar binary\n".to_vec();
        bytes.extend_from_slice(&crate::path::MORTAR_MARKER);
        executable(&exe, &bytes);
        let env = InstallEnv {
            path: Some(std::env::join_paths([home.join(".local/bin")]).unwrap()),
            home,
            cwd: project.clone(),
            project,
            exe: Some(exe),
            prefixes: vec![prefix],
            terminal: false,
        };
        Self { root, env }
    }

    pub fn path(&mut self, entries: &[&Path]) {
        self.env.path = Some(std::env::join_paths(entries).unwrap());
    }

    pub fn install(&self, options: &InstallOptions) -> Report {
        install_in(&self.env, options, &mut |_| panic!("unexpected picker"))
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

pub(super) struct RestorePermissions(PathBuf);

impl Drop for RestorePermissions {
    fn drop(&mut self) {
        fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

pub(super) fn deny_writes(dir: PathBuf) -> RestorePermissions {
    let restore = RestorePermissions(dir.clone());
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
    let probe = dir.join("probe");
    match fs::write(&probe, b"probe") {
        Ok(()) => {
            fs::remove_file(probe).unwrap();
            panic!("test needs a non-root user to deny writes to the folder");
        }
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied),
    }
    restore
}

pub(super) fn options() -> InstallOptions {
    InstallOptions {
        no_place: false,
        short_name: false,
        targets: None,
        yes: false,
        location: SkillLocation::Global,
    }
}

pub(super) fn executable(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

pub(super) fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, dir: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let kind = fs::symlink_metadata(&path).unwrap().file_type();
            let value = if kind.is_symlink() {
                Entry::Link(fs::read_link(&path).unwrap())
            } else if kind.is_dir() {
                visit(root, &path, entries);
                Entry::Directory
            } else {
                assert!(kind.is_file());
                Entry::File(fs::read(&path).unwrap())
            };
            entries.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

pub(super) fn assert_report(report: &Report, exit: i32, outcome: &str) {
    assert_eq!(report.exit as i32, exit, "{report:?}");
    assert_eq!(report.outcome, outcome, "{report:?}");
}
