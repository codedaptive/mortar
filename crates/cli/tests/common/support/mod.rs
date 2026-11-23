use crate::json::{check, Report};
mod assert_values_then_bases;
mod fixture_config;
pub mod fixture_partners;
mod fixture_formatter;
mod fixture_plain;
mod fixture_zone;
mod write_fixture_config;
// Only resolve_runs and across_kits call it; the includers' dead_code allow does not cover an unused re-export.
#[allow(unused_imports)]
pub use assert_values_then_bases::assert_values_then_bases;
pub use fixture_config::FixtureConfig as Config;
pub use fixture_formatter::FixtureFormatter as FormatterConfig;
pub use fixture_zone::FixtureZone as ZoneConfig;
pub use write_fixture_config::write_fixture_config;
use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::{Git, Oid, RunOpts};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName};
use std::{collections::BTreeMap, ffi::OsString, fs, os::unix::fs::PermissionsExt, path::{Path, PathBuf}, process::{Command, Output, Stdio}, sync::atomic::{AtomicU64, Ordering}};

pub const REQUEST: &str = "refs/heads/req/core/task";
pub const ZONE: &str = "refs/heads/zone/core";
pub const SNAP: &str = "refs/mortar/snap/core/task";

pub struct Fixture {
    pub root: PathBuf, pub git: Git, pub common: PathBuf, pub request: PathBuf, pub zone: PathBuf,
    pub base: Oid, pub init_args: Config, _owned: OwnDir,
}
impl Fixture {
    pub fn new() -> Self {
        let (owned, git, base) = main_repo(); let root = owned.0.clone(); let common = git.common_dir().unwrap();
        let request = root.join("request"); let zone = root.join("zone"); let paths = zone_paths();
        oplog::recover(&git, &common).unwrap();
        folders::make_zone_folder(&setup(&git, &zone), "core", &paths, Some(&base)).unwrap();
        oplog::recover(&git, &common).unwrap();
        folders::make_request_folder(&setup(&git, &request), "core", "task", &paths).unwrap();
        let init_args = configure_main(&git);
        Self { root, git, common, request, zone, base, init_args, _owned: owned }
    }
    pub fn command(&self, cwd: &Path, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_mortar"));
        c.current_dir(cwd).env_clear().envs(self.git.env.iter().cloned())
            .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", &self.root).env("TMPDIR", &self.root)
            .env("CODETEAM_SESSION", "harness").args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        c
    }
    pub fn app(&self, args: &[&str]) -> Output { self.command(&self.request, args).output().unwrap() }
    pub fn report(&self, args: &[&str], verb: &str, exit: i32, outcome: &str) -> Report { check(self.app(args), verb, exit, outcome) }
    pub fn configure(&self, config: &Config) { write_fixture_config(&self.git.repo, config).unwrap(); }
    /// Read the binary's status; each caller pins the relevant public fields and keeps its raw-state oracle.
    pub fn status(&self) -> Report { crate::json::read(self.app(&["--json", "status"]), "status", 0) }
    /// Audit the last real Done record moving this fixture's request, without cli's pending-move detector.
    pub fn last_move(&self) -> oplog::ValuedRecord {
        oplog::read_valued_log(&self.common).unwrap().records.into_iter().rev().find(|r| {
            r.record.phase == oplog::OpPhase::Done && r.record.moves.iter().any(|m| m.refname == REQUEST)
        }).expect("a real request move in this fixture")
    }
    pub fn local(&self) -> Git { Git { repo: self.request.clone(), ..self.git.clone() } }
    pub fn head(&self) -> Oid { self.git.read_ref(REQUEST).unwrap().unwrap() }
    pub fn records(&self) -> Vec<oplog::OpRecord> { oplog::read_op_log(&self.common).unwrap().records }
    pub fn log(&self) -> Option<Vec<u8>> { fs::read(oplog::oplog_path(&self.common)).ok() }
    pub fn lock_path(&self) -> PathBuf { self.common.join("mortar/locks/join-core.lock") }
    pub fn write(&self, path: &str, bytes: &[u8]) { write(&self.request, path, bytes); }
    pub fn blob(&self, object: &Oid, path: &str) -> Vec<u8> { raw(&self.git, &["show", &format!("{object}:{path}")]) }
    pub fn save_topic(&self) -> Oid {
        self.write("src/file", b"topic work\n"); self.report(&["--json", "save", "Save topic"], "save", 0, "saved"); self.head()
    }
    pub fn advance_zone(&self, conflict: bool) -> Oid {
        let edits: &[(&str, &[u8])] = if conflict { &[("src/file", b"zone work\n")] } else { &[("src/upstream", b"upstream work\n")] };
        let next = self.commit(&self.base, edits, "zone advanced\n");
        raw(&self.git, &["update-ref", ZONE, next.as_str()]); next
    }
    pub fn commit(&self, parent: &Oid, edits: &[(&str, &[u8])], message: &str) -> Oid {
        let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), self.common.join("validation-oracle-index").into_os_string())], ..RunOpts::default() };
        self.git.run(&["read-tree", parent.as_str()], &opts).unwrap();
        for (path, bytes) in edits {
            let id = oid(&self.git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap().stdout);
            self.git.run(&["update-index", "--add", "--cacheinfo", "100644", id.as_str(), path], &opts).unwrap();
        }
        let tree = oid(&self.git.run(&["write-tree"], &opts).unwrap().stdout);
        self.git.commit_tree(&tree, std::slice::from_ref(parent), message, None, None).unwrap()
    }
    pub fn pending_rebase(&self) -> (Oid, Oid, Oid) {
        let old = self.save_topic(); let zone = self.advance_zone(false);
        let answer = with_lock(&self.git, &self.common, &LockName::Join("core".into()), "validation-setup-rebase", &KillPidProbe, || {
            oplog::recover(&self.git, &self.common).map_err(join::JoinRunError::Oplog)?;
            join::join(&self.git, &self.request, "core", REQUEST, "core/task")
        });
        let answer = match answer { Ok(a) => a, Err(HeldLockError::Work(e)) => panic!("setup rebase: {e:?}"), Err(e) => panic!("setup lock: {e:?}") };
        let join::JoinAnswer::Rebased { from, to, folder_tree: None, .. } = answer else { panic!("expected plain rebase") };
        assert_eq!(from, old); assert!(!self.request.join("src/upstream").exists());
        (old, to, zone)
    }
    pub fn observe(&self) -> (Vec<u8>, Vec<u8>, Option<Vec<u8>>, Vec<u8>) {
        (raw(&self.git, &["for-each-ref", "--format=%(refname) %(objectname)"]),
         raw(&self.local(), &["ls-files", "--stage", "-z"]), self.log(), fs::read(self.request.join("src/file")).unwrap())
    }
}
/// The isolated git environment and the main repository with its base commit, shared by every constructor.
fn main_repo() -> (OwnDir, Git, Oid) {
    let owned = OwnDir::new(); let main = owned.0.join("main");
    let mut git = Git::new(&owned.0);
    git.env = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ("GIT_AUTHOR_NAME".into(), "CLI Validation Author".into()), ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_AUTHOR_DATE".into(), "1700000000 +0000".into()),
        ("GIT_COMMITTER_NAME".into(), "CLI Validation Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ("GIT_COMMITTER_DATE".into(), "1700000100 +0000".into()),
    ];
    git.run(&["init", "-q", "--initial-branch=main", main.to_str().unwrap()], &RunOpts::default()).unwrap();
    git.repo = main;
    write(&git.repo, "root.txt", b"root\n"); write(&git.repo, "src/file", b"base\n"); write(&git.repo, "sub/keep", b"keep\n");
    raw(&git, &["add", "-A"]); raw(&git, &["commit", "-qm", "base"]);
    let base = git.rev_parse("HEAD").unwrap().unwrap();
    (owned, git, base)
}
fn zone_paths() -> Vec<String> { vec!["src".into(), "sub".into()] }
/// Write the fixture's config (zone "core" over src and sub) into the main repository and return it.
fn configure_main(git: &Git) -> Config {
    let config = Config { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
        zones: BTreeMap::from([("core".into(), ZoneConfig { paths: zone_paths() })]), root_files: vec!["root.txt".into()],
        build_dirs: vec!["d/".into()], cap_bytes: snapshot::DEFAULT_SIZE_CAP, floor_bytes: 0, formatter: None };
    write_fixture_config(&git.repo, &config).unwrap();
    config
}
pub fn setup<'a>(git: &'a Git, path: &Path) -> FolderSetup<'a> {
    FolderSetup { git, path: path.into(), floor_bytes: 0, sweep: SweepEstimate { bytes: 0, reason: None }, disk: &DfFreeDisk, pids: &KillPidProbe, actor: "validation-fixture".into() }
}
pub fn raw(git: &Git, args: &[&str]) -> Vec<u8> { git.run(args, &RunOpts::default()).unwrap().stdout }
pub fn oid(bytes: &[u8]) -> Oid { Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap() }
pub fn write(root: &Path, name: &str, bytes: &[u8]) { let path = root.join(name); fs::create_dir_all(path.parent().unwrap()).unwrap(); fs::write(path, bytes).unwrap(); }
pub fn script(path: &Path, bytes: &[u8]) { fs::write(path, bytes).unwrap(); fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap(); }
pub fn actual_git() -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").unwrap()).map(|p| p.join("git")).find(|p| p.is_file()).unwrap().canonicalize().unwrap()
}
pub fn shim_path(dir: &Path) -> OsString { std::env::join_paths(std::iter::once(dir.to_path_buf()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap()))).unwrap() }
pub struct OwnDir(pub PathBuf);
impl OwnDir {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!("mortar-cli-validation-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            match fs::create_dir(&path) { Ok(()) => return Self(path.canonicalize().unwrap()), Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue, Err(e) => panic!("temp directory: {e}") }
        }
    }
}
impl Drop for OwnDir { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
