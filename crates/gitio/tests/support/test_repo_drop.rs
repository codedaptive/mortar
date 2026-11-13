use super::TestRepo;

impl Drop for TestRepo {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            // Do not mask an assertion failure with a second panic during unwinding.
            if std::thread::panicking() {
                eprintln!("remove test repository {}: {error}", self.path.display());
            } else {
                panic!("remove test repository {}: {error}", self.path.display());
            }
        }
    }
}
