use super::{run, write, Fixture};
use gitio::{Git, Oid};
use testkit::TestRepo;

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Folder Fixture".into()),
            ("GIT_AUTHOR_EMAIL".into(), "fixture@example.invalid".into()),
            ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
            ("GIT_COMMITTER_NAME".into(), "Folder Fixture".into()),
            ("GIT_COMMITTER_EMAIL".into(), "fixture@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "2000-01-02T00:00:00 +0000".into()),
        ];
        for (path, content) in [
            ("Cargo.toml", "# root build file\n"), ("README", "root file\n"),
            ("kit/zone/file", "zone\n"), ("kit/zone/conflict", "base\n"),
            ("kit/dependency/file", "dependency\n"), ("needed/file", "needed\n"), ("other/file", "other\n"),
        ] { write(&repo.path, path, content.as_bytes()); }
        run(&git, &["add", "-A"]);
        run(&git, &["commit", "-m", "folder fixture"]);
        let head = Oid::parse(std::str::from_utf8(&run(&git, &["rev-parse", "HEAD"])).unwrap().trim()).unwrap();
        let common = git.common_dir().unwrap();
        assert!(oplog::recover(&git, &common).unwrap().is_empty());
        Self { repo, git, head, common }
    }
}
