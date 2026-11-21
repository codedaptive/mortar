//! create_with_redraw: draw a path and open it, drawing again while the drawn one is already taken.
//! Pure: it calls only the two closures it is given. create_with_redraw_tests drives it with closures alone.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::GuardError;

/// Names drawn before giving up. Each name is drawn at random, so one already taken means a crash-left file or a
/// hostile writer; eight collisions in a row is no accident, and the caller is better told than kept looping.
const CREATE_ATTEMPTS: usize = 8;

/// Draws a path with `draw` and opens it with `open`, returning the path and what `open` gave. An open that fails
/// with `AlreadyExists` draws again, up to `CREATE_ATTEMPTS` draws in all. Any other failure, or `AlreadyExists`
/// on the last draw, is `GuardError::Io` naming the path drawn for that open.
pub(crate) fn create_with_redraw<T>(
    mut draw: impl FnMut() -> PathBuf,
    mut open: impl FnMut(&Path) -> io::Result<T>,
) -> Result<(PathBuf, T), GuardError> {
    let mut attempt = 1;
    loop {
        let path = draw();
        match open(&path) {
            Ok(value) => return Ok((path, value)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists && attempt < CREATE_ATTEMPTS => attempt += 1,
            Err(error) => return Err(GuardError::Io { path, error }),
        }
    }
}
