use super::{run, write, Fixture};
use gitio::{Git, Oid};
use oplog::{acquire_lock, host_name, recover, KillPidProbe, LockName, LockRecord};
use snapshot::SnapshotTarget;
use testkit::TestRepo;

impl Fixture {
    pub fn new(files: &[(&str, &[u8])]) -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Fixture Author".into()),
            ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
            ("GIT_COMMITTER_NAME".into(), "Fixture Committer".into()),
            ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "2000-01-02T00:00:00 +0000".into()),
        ];
        write(&repo.path, "root.txt", b"base\n");
        for (path, bytes) in files { write(&repo.path, path, bytes); }
        run(&git, &["add", "-A"]);
        run(&git, &["commit", "-m", "fixture"]);
        let head = Oid::parse(std::str::from_utf8(&run(&git, &["rev-parse", "HEAD"])).unwrap().trim()).unwrap();
        let common = git.common_dir().unwrap();
        let target = SnapshotTarget { zone: "snapshot-zone".into(), request: "request-17".into(), actor: "snapshot-caller".into() };
        let refname = "refs/mortar/snap/snapshot-zone/request-17".into();
        let holder = LockRecord { pid: std::process::id(), host: host_name().unwrap(), key: "snapshot-integration".into(), since: 100 };
        let lock = acquire_lock(&common, &LockName::Join(target.zone.clone()), &holder, &KillPidProbe).unwrap();
        assert!(recover(&git, &common).unwrap().is_empty());
        Self { repo, git, head, common, target, refname, lock }
    }
}
