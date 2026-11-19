//! file_has_marker: whether a file's bytes carry a marker (is the first `mr` on PATH a copy of Mortar). It
//! reads the file and never runs it. No sibling test: its job is reading files. marker_in_chunks_tests
//! cover the search, the 64 KiB straddle included. Its real-file cases (marker present or absent, straddling a read,
//! a directory, a missing path, a symlink) are listed for the validation tests through `mr guard install`.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::marker_in_chunks;

/// One read: 64 KiB.
const CHUNK: usize = 64 * 1024;
/// The most read from one file: 256 MiB, well past any Mortar binary, so a huge or endless file ends the scan.
const BOUND: u64 = 256 * 1024 * 1024;

/// True when `path` is a regular file (following links) and `marker` occurs in its first 256 MiB, read 64 KiB at a
/// time. A marker found before a read error stops the scan is still found. Not a regular file, missing, unreadable,
/// or the marker not met before the end, a read error or the bound: false.
pub fn file_has_marker(path: &Path, marker: &[u8]) -> bool {
    if !fs::metadata(path).is_ok_and(|meta| meta.is_file()) {
        return false;
    }
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut reader = file.take(BOUND);
    let chunks = std::iter::from_fn(move || {
        let mut buffer = vec![0; CHUNK];
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => None,
            Ok(read) => {
                buffer.truncate(read);
                Some(buffer)
            }
        }
    });
    marker_in_chunks(chunks, marker)
}
