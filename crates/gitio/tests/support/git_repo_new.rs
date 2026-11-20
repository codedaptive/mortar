use super::GitRepo;
use gitio::Git;

impl GitRepo {
    pub fn new() -> Self {
        let repository = testkit::TestRepo::new().unwrap();
        let mut git = Git::new(&repository.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ];
        Self { git, repository }
    }
}
