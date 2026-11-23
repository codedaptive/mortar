use super::{oid, raw, write, write_fixture_config, Config, ZoneConfig, REQUEST, ZONE};
use gitio::{Git, Oid, RunOpts};
use std::{collections::BTreeMap, path::Path, process::{Command, Output, Stdio}};
use testkit::TestRepo;

pub struct Partners {
    pub repo: TestRepo,
    pub git: Git,
    pub base: Oid,
    pub web: Oid,
    pub request: std::path::PathBuf,
}

impl Partners {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Test Author".into()), ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_COMMITTER_NAME".into(), "Test Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ];
        write(&repo.path, "Cargo.toml", b"[workspace]\n");
        write(&repo.path, "src/core/c.txt", b"core base\n");
        write(&repo.path, "web/w.txt", b"web base\n");
        raw(&git, &["add", "-A"]); raw(&git, &["commit", "-qm", "base"]);
        let base = git.rev_parse("HEAD").unwrap().unwrap();
        raw(&git, &["branch", "zone/core", base.as_str()]);
        raw(&git, &["branch", "zone/web", base.as_str()]);
        write_fixture_config(&git.repo, &Config {
            origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into(),
            zones: BTreeMap::from([
                ("core".into(), ZoneConfig { paths: vec!["src/core".into()] }),
                ("web".into(), ZoneConfig { paths: vec!["web".into()] }),
            ]), root_files: vec!["Cargo.toml".into()], build_dirs: vec![],
            cap_bytes: snapshot::DEFAULT_SIZE_CAP, floor_bytes: 0, formatter: None,
        }).unwrap();
        let request = repo.path.join("request");
        let mut f = Self { repo, git, base: base.clone(), web: base, request };
        f.web = f.advance("refs/heads/zone/web", &[("web/w.txt", b"web partner\n")]);
        f
    }
    pub fn command(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(cwd).env_clear().envs(self.git.env.iter().cloned())
            .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", &self.repo.path).env("TMPDIR", &self.repo.path)
            .env("CODETEAM_SESSION", "harness").stdin(Stdio::null()).args(args).output().unwrap()
    }
    pub fn cut(&self, extra: &[&str]) -> Output {
        let mut args = vec!["--actor", "harness", "--json", "folder", "request", "core", "task", self.request.to_str().unwrap(), "--paths", "src/core"];
        args.extend_from_slice(extra);
        self.command(&self.repo.path, &args)
    }
    pub fn set(&self) -> Oid {
        let out = self.cut(&["--with", "web"]);
        crate::json::check(out, "folder", 0, "made");
        self.tip(REQUEST)
    }
    pub fn tip(&self, name: &str) -> Oid { self.git.read_ref(name).unwrap().unwrap() }
    pub fn local(&self) -> Git { Git { repo: self.request.clone(), ..self.git.clone() } }
    pub fn advance(&self, branch: &str, edits: &[(&str, &[u8])]) -> Oid {
        let parent = self.tip(branch);
        let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), self.git.common_dir().unwrap().join("partners-index").into_os_string())], ..RunOpts::default() };
        self.git.run(&["read-tree", parent.as_str()], &opts).unwrap();
        for (path, bytes) in edits {
            let blob = oid(&self.git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap().stdout);
            self.git.run(&["update-index", "--add", "--cacheinfo", "100644", blob.as_str(), path], &opts).unwrap();
        }
        let tree = oid(&self.git.run(&["write-tree"], &opts).unwrap().stdout);
        let next = self.git.commit_tree(&tree, &[parent], "advance\n", None, None).unwrap();
        raw(&self.git, &["update-ref", branch, next.as_str()]);
        next
    }
    pub fn commit_request(&self, path: &str, bytes: &[u8]) {
        write(&self.request, path, bytes);
        raw(&self.local(), &["add", "--sparse", path]);
        raw(&self.local(), &["commit", "-qm", "request"]);
    }
    pub fn app(&self, verb: &str) -> Output { self.command(&self.request, &["--json", verb]) }
    pub fn zone(&self) -> Oid { self.tip(ZONE) }
}

pub fn text(out: &Output) -> String { String::from_utf8(out.stdout.clone()).unwrap() }
pub fn partners(tip: &Oid) -> String { format!("\"partners\":[{{\"zone\":\"web\",\"tip\":\"{tip}\"}}]") }
