use gitio::{Git, Oid, RunOpts};
use std::path::PathBuf;
use testkit::TestRepo;

pub const CHANGE_ID: &str = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
pub struct Fixture { pub git: Git, pub common: PathBuf, pub source: Oid, _repo: TestRepo }
impl Fixture {
    pub fn new() -> Self {
        // TestRepo's own init also sets GLOBAL=/dev/null and NOSYSTEM=1 (test_repo_new.rs21-25).
        let repo = TestRepo::new().unwrap(); let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("LC_ALL".into(), "C".into()), ("GIT_AUTHOR_NAME".into(), "Land RIT".into()),
            ("GIT_AUTHOR_EMAIL".into(), "land@example.invalid".into()), ("GIT_AUTHOR_DATE".into(), "1700000000 +0000".into()),
            ("GIT_COMMITTER_NAME".into(), "Land RIT".into()), ("GIT_COMMITTER_EMAIL".into(), "land@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "1700000100 +0000".into()),
        ];
        let base_tree = tree(&git, b"base\n");
        let base = git.commit_tree(&base_tree, &[], "base\n", None, None).unwrap();
        let source_tree = tree(&git, b"zone change\n");
        let source = git.commit_tree(&source_tree, &[base.clone()],
            &format!("request change\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Request: task\nMortar-Zone: core\n"), None, None).unwrap();
        // A literal known prior ID makes the trailer expectation independent of random new-ID generation.
        let prior = git.commit_tree(&base_tree, &[base.clone()],
            &format!("prior squash\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Zone: core\nMortar-Squash-Of: {base}\n"), None, None).unwrap();
        for (name, tip) in [("refs/heads/main", &base), ("refs/heads/integration", &base),
            ("refs/heads/zone/core", &source), ("refs/mortar/squash/core", &prior)] {
            git.run(&["update-ref", name, tip.as_str()], &RunOpts::default()).unwrap();
        }
        let common = git.common_dir().unwrap(); Self { git, common, source, _repo: repo }
    }
}
fn tree(git: &Git, bytes: &[u8]) -> Oid {
    let blob = oid(&git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap().stdout);
    let input = format!("100644 blob {blob}\tfile\n");
    oid(&git.run(&["mktree"], &RunOpts { stdin: Some(input.into_bytes()), ..RunOpts::default() }).unwrap().stdout)
}
fn oid(bytes: &[u8]) -> Oid { Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap() }
