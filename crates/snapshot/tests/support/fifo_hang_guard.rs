use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf, sync::mpsc, thread, time::Duration};

/// Failure-only watchdog: marks its intervention and releases fixture FIFOs so a kill regression can return and fail.
/// It is cancelled/joined on Drop; no outcome is inferred from elapsed time.
pub fn fifo_hang_guard(fifos: Vec<PathBuf>, fired: PathBuf, limit: Duration) -> impl Drop {
    struct Guard { cancel: mpsc::Sender<()>, thread: Option<thread::JoinHandle<()>> }
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.cancel.send(());
            let result = self.thread.take().unwrap().join();
            if !thread::panicking() { result.unwrap(); }
        }
    }
    let (cancel, receiver) = mpsc::channel();
    let thread = thread::spawn(move || {
        if receiver.recv_timeout(limit) == Err(mpsc::RecvTimeoutError::Timeout) {
            fs::write(fired, b"hang guard released fixture\n").unwrap();
            for fifo in fifos {
                // Read/write open never waits for another endpoint on the Unix targets. A small write
                // wakes a stalled reader even if the formatter never started or has already died.
                let mut file = OpenOptions::new().read(true).write(true).open(fifo).unwrap();
                file.write_all(b"WATCHDOG\n").unwrap();
            }
        }
    });
    Guard { cancel, thread: Some(thread) }
}
