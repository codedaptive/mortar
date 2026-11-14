use gitio::{Git, Oid, RunOpts};
use join::{JoinAnswer, JoinRunError};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName};
use std::{fs, path::PathBuf};
use testkit::{write_git_shim, TestRepo};

const REQUEST: &str = "refs/heads/requests/rit";
const FILES: [&str; 5] = ["vendor/lib/file", "lib/x", "src/a.rs", "my \"odd\" file.txt", "ok.txt"];

pub(super) struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub tree: Oid,
    pub tip: Oid,
    base: Oid,
}

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env.extend([
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_AUTHOR_NAME".into(), "RIT Author".into()), ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_AUTHOR_DATE".into(), "1700000000 +0000".into()),
            ("GIT_COMMITTER_NAME".into(), "RIT Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "1700000100 +0000".into()),
        ]);
        for name in FILES {
            let path = repo.path.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, format!("keep {name}\n")).unwrap();
        }
        git.run(&["add", "--", "."], &RunOpts::default()).unwrap();
        let tree = oid(&git.run(&["write-tree"], &RunOpts::default()).unwrap().stdout);
        let base = git.commit_tree(&tree, &[], "base\n", None, None).unwrap();
        let tip = git.commit_tree(&tree, &[base.clone()], "request\n", None, None).unwrap();
        git.run(&["update-ref", "refs/heads/main", base.as_str()], &RunOpts::default()).unwrap();
        git.run(&["update-ref", REQUEST, tip.as_str()], &RunOpts::default()).unwrap();
        git.run(&["reset", "--mixed", "main"], &RunOpts::default()).unwrap();
        Self { repo, git, tree, tip, base }
    }

    pub fn state(&self, files: &str, all_files: &str, written: bool) {
        // Controlled literal JSON arrays are supplied by tests; no production encoder is used as an oracle.
        let text = format!(
            "{{\"request_ref\":\"{REQUEST}\",\"request_tip\":\"{}\",\"onto\":\"{}\",\"replayed_tip\":\"{}\",\"commit\":\"{}\",\"tree\":\"{}\",\"remaining\":[],\"files\":{files},\"all_files\":{all_files},\"files_written\":{written},\"rebased_to\":null,\"folder_tree\":null}}\n",
            self.tip, self.base, self.base, self.tip, self.tree,
        );
        fs::write(self.repo.path.join(".git/mortar-join.json"), text).unwrap();
    }

    pub fn runner(&self, stdout: &[u8], stderr: &[u8], code: Option<i32>) -> Git {
        let dir = self.repo.path.join(".git");
        fs::write(dir.join("reply-out"), stdout).unwrap();
        fs::write(dir.join("reply-err"), stderr).unwrap();
        let shim = dir.join("rit-git.sh");
        // Real git handles everything except the explicitly selected ls-tree wire/failure fixture.
        // The trace independently proves where resolve stopped, including no tree/ref writes on early refusal.
        write_git_shim(&shim, b"#!/bin/sh\nset -eu\nfor arg in \"$@\"; do\n  case \"$arg\" in ls-tree|read-tree|update-index|write-tree|commit-tree|update-ref) printf '%s\\n' \"$arg\" >>\"$JOIN_RIT_TRACE\";; esac\ndone\nif [ \"${3-}\" = ls-tree ] && [ \"$JOIN_RIT_INJECT\" = 1 ]; then\n  cat \"$JOIN_RIT_OUT\"\n  cat \"$JOIN_RIT_ERR\" >&2\n  exit \"$JOIN_RIT_CODE\"\nfi\nexec git \"$@\"\n").unwrap();
        let mut git = Git { program: shim, ..self.git.clone() };
        git.env.extend([
            ("JOIN_RIT_TRACE".into(), dir.join("rit-trace").into_os_string()),
            ("JOIN_RIT_OUT".into(), dir.join("reply-out").into_os_string()),
            ("JOIN_RIT_ERR".into(), dir.join("reply-err").into_os_string()),
            ("JOIN_RIT_INJECT".into(), if code.is_some() { "1" } else { "0" }.into()),
            ("JOIN_RIT_CODE".into(), code.unwrap_or(0).to_string().into()),
        ]);
        git
    }

    pub fn resolve(&self, git: &Git) -> Result<JoinAnswer, JoinRunError> {
        let common = self.repo.path.join(".git");
        match with_lock(git, &common, &LockName::Join("rit".into()), "join-rit", &KillPidProbe, || {
            oplog::recover(git, &common).map_err(JoinRunError::Oplog)?;
            join::resolve(git, &self.repo.path, &self.tree, "join-rit")
        }) {
            Ok(answer) => Ok(answer),
            Err(HeldLockError::Work(error)) => Err(error),
            Err(error) => panic!("caller lock: {error:?}"),
        }
    }

    pub fn snapshot(&self) -> Vec<Vec<u8>> {
        let mut result = vec![
            self.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout,
            self.git.run(&["rev-parse", "HEAD"], &RunOpts::default()).unwrap().stdout,
            fs::read(self.repo.path.join(".git/index")).unwrap(),
            fs::read(self.repo.path.join(".git/mortar-join.json")).unwrap(),
            fs::read(oplog::oplog_path(&self.repo.path.join(".git"))).unwrap_or_default(),
        ];
        result.extend(FILES.iter().map(|name| fs::read(self.repo.path.join(name)).unwrap()));
        result
    }

    pub fn unchanged(&self, before: &[Vec<u8>]) {
        assert_eq!(self.snapshot(), before, "state, refs, HEAD, index, log and existing files must survive refusal");
        assert!(!self.repo.path.join(".git/mortar-join.json.tmp").exists());
    }

    pub fn trace(&self) -> String {
        fs::read_to_string(self.repo.path.join(".git/rit-trace")).unwrap_or_default()
    }
}

fn oid(bytes: &[u8]) -> Oid { Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap() }

pub(super) fn path_refused(error: JoinRunError) -> (Option<PathBuf>, String) {
    match error {
        JoinRunError::ConflictedPathRefused { path, reason } => (path, reason),
        other => panic!("expected ConflictedPathRefused, got {other:?}"),
    }
}
