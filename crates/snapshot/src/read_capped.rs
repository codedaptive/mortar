//! read_capped: drain a command's stderr to EOF, keeping only its first FORMATTER_STDERR_CAP bytes.

use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use std::io::{self, Read};
use std::sync::{Mutex, PoisonError};

/// Reads `pipe` to EOF in chunks, keeping its first FORMATTER_STDERR_CAP bytes in `kept` and dropping the rest.
pub(crate) fn read_capped<R: Read>(mut pipe: R, kept: &Mutex<Vec<u8>>) -> io::Result<()> {
    let mut chunk = [0u8; 8192];
    loop {
        let read = match pipe.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(read) => read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        let mut kept = kept.lock().unwrap_or_else(PoisonError::into_inner);
        let room = FORMATTER_STDERR_CAP.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..read.min(room)]);
    }
}
