use super::TestRepo;
use gitio::{Git, RunOpts};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

impl TestRepo {
    pub fn new() -> Self {
        let path = loop {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mortar-gitio-it-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create {}: {error}", path.display()),
            }
        };
        let mut git = Git::new(&path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ];
        // Construct the guard before init, so even a failed init removes its directory.
        let repo = Self { path, git };
        let output = repo.git.run(&["init", "-b", "main"], &RunOpts::default()).unwrap();
        assert_eq!(output.status, 0);
        assert!(repo.path.join(".git/HEAD").is_file());
        repo
    }
}
