use super::Fixture;

impl Drop for Fixture {
    fn drop(&mut self) {
        // Release before TestRepo removes its directory, without a second panic during unwinding.
        let result = oplog::release_lock(&self.lock);
        if !std::thread::panicking() { result.unwrap(); }
    }
}
