use crate::{cid::CID, refs::refs, support::{actual_git, raw, shim_path, Fixture}};
use gitio::{Git, Oid};
use std::{fs, io::Write, os::unix::fs::symlink, path::PathBuf, process::{Command, Output, Stdio}};

pub struct Guard { pub f: Fixture, pub new: Oid, pub origin: PathBuf, pub other: PathBuf }
impl Guard {
    pub fn new() -> Self {
        let mut f = Fixture::new();
        let new = f.commit(&f.base, &[("src/file", b"guarded published work\n")],
            &format!("guard fixture\n\nMortar-Change-Id: {CID}\nMortar-Zone: core\n"));
        let origin = f.root.join("origin.git"); let other = f.root.join("other.git");
        for path in [&origin, &other] {
            raw(&f.git, &["init", "--bare", "--initial-branch=main", path.to_str().unwrap()]);
            raw(&f.git, &["push", path.to_str().unwrap(), &format!("{}:refs/heads/main", f.base)]);
        }
        raw(&f.git, &["remote", "add", "origin", origin.to_str().unwrap()]);
        raw(&f.git, &["remote", "add", "other", other.to_str().unwrap()]);
        raw(&f.git, &["update-ref", "refs/heads/integration", f.base.as_str()]);
        let bin = f.root.join("hook-bin"); fs::create_dir(&bin).unwrap();
        symlink(env!("CARGO_BIN_EXE_mortar"), bin.join("mortar")).unwrap();
        f.git.env.push(("PATH".into(), shim_path(&bin)));
        f.git.program = actual_git();
        // S-112: actual product installer and its untouched executable hook scripts, not a fake hook.
        land::guard_install(&f.git, &refs()).unwrap();
        Self { f, new, origin, other }
    }
    pub fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(actual_git());
        cmd.arg("-C").arg(&self.f.git.repo).args(args).env_clear().envs(self.f.git.env.iter().cloned())
            .env("HOME", &self.f.root).env("TMPDIR", &self.f.root).stdin(Stdio::null())
            .stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }
    pub fn output(&self, args: &[&str]) -> Output { self.command(args).output().unwrap() }
    pub fn remote(&self, other: bool) -> Option<Oid> {
        let git = Git { repo: if other { self.other.clone() } else { self.origin.clone() }, ..self.f.git.clone() };
        git.read_ref("refs/heads/main").unwrap()
    }
    pub fn branches(&self) -> Vec<u8> { raw(&self.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]) }
    pub fn hook(&self, words: &[&str], input: &[u8]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mortar"));
        cmd.current_dir(&self.f.git.repo).args(words).env_clear().envs(self.f.git.env.iter().cloned())
            .env("HOME", &self.f.root).env("TMPDIR", &self.f.root).stdin(Stdio::piped())
            .stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().unwrap(); child.stdin.take().unwrap().write_all(input).unwrap();
        child.wait_with_output().unwrap()
    }
    pub fn no_move(&self, before: &[u8], output: Output, reason: &str) {
        assert!(!output.status.success(), "guard must refuse: {output:?}");
        // Git forwards pre-push's report on stdout, but transaction-hook reports may be in stderr.
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(text.contains(reason), "{output:?}");
        assert_eq!(self.branches(), before); assert_eq!(self.remote(false), Some(self.f.base.clone()));
    }
}
