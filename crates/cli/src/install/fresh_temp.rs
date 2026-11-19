//! fresh_temp: Creates a sibling temp under a random name through an exclusive `make`, moving on from any name already
//! taken, so nothing already at a name is followed or replaced.
//! Pure over its two closures: it touches nothing itself; `make` does any disk work. fresh_temp_tests pins its tries
//! with fakes; the integration tests in install/integration/temp_files.rs run it against real files.

use std::io;
use std::path::{Path, PathBuf};

use super::temp_path;

/// How many names are tried before giving up: with random tokens a taken name is already unlikely, so eight taken in
/// a row means something is planting names, not chance.
const TRIES: usize = 8;

/// Up to 8 tries: `make` creates the temp at `temp_path(path, token())` exclusively; a name already taken moves on to
/// the next token. Returns the temp's path and what `make` returned, `make`'s first other error, or an AlreadyExists
/// error naming `path` when all 8 names are taken. `make` only creates the temp; the caller writes into it and
/// removes it on a later error.
pub fn fresh_temp<T>(
    path: &Path,
    token: &mut dyn FnMut() -> u64,
    make: &mut dyn FnMut(&Path) -> io::Result<T>,
) -> io::Result<(PathBuf, T)> {
    for _ in 0..TRIES {
        let temp = temp_path(path, token());
        match make(&temp) {
            Ok(value) => return Ok((temp, value)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("every temp name tried beside {} is taken", path.display()),
    ))
}
