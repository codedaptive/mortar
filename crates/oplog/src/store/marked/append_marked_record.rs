//! Appends one record with its values and remote mark to the op log and syncs it to disk (S-1).
//! The I/O only. No unit test of its own: it writes a real file (S-5).
//! crates/oplog/tests/oplog/marked_lines.rs covers the exact bytes it writes, including the append after a
//! torn tail (a file not ending in '\n') that starts on a new line. Nagatha's integration group should
//! cover: the first append creates `<common>/mortar/` and the file; each append adds exactly one line that
//! read_marked_log returns with its values and mark; an unwritable folder is an Io error naming the path.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::store::{log_line, oplog_path, OpRecord, OplogError};

/// Writes log_line(r, values, remote, the file's last byte) with O_APPEND in one write, fsyncs the file,
/// then fsyncs the `<common>/mortar/` folder.
pub(crate) fn append_marked_record(
    common: &Path,
    r: &OpRecord,
    values: &[(String, String)],
    remote: Option<&str>,
) -> Result<(), OplogError> {
    let path = oplog_path(common);
    let folder = path.parent().unwrap_or(common);
    std::fs::create_dir_all(folder).map_err(|source| io(folder.to_path_buf(), source))?;
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|source| io(path.clone(), source))?;
    let last = last_byte(&mut file).map_err(|source| io(path.clone(), source))?;
    file.write_all(log_line(r, values, remote, last).as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| io(path.clone(), source))?;
    // A file's fsync does not make its directory entry durable: without syncing the folder, a log file
    // created by this append could vanish in a crash. Done on every append; it is one cheap call.
    File::open(folder)
        .and_then(|dir| dir.sync_all())
        .map_err(|source| io(folder.to_path_buf(), source))
}

/// The file's last byte, None when it is empty.
fn last_byte(file: &mut File) -> std::io::Result<Option<u8>> {
    if file.metadata()?.len() == 0 {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    Ok(Some(last[0]))
}

fn io(path: PathBuf, source: std::io::Error) -> OplogError {
    OplogError::Io { path, source }
}
