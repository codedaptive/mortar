//! What one `mr folder` line asks for: its sub-verb and that sub-verb's words.
//! A bare type; parse_folder_args_tests exercises it.

use std::path::PathBuf;

/// `Zone`: make or refill `zone`'s folder at `path`, its branch started at `start` when it does not exist yet.
/// `Request`: make request `request`'s folder at `path`, cut from `zone`'s branch, on `paths`. `Remove`: remove the
/// folder at `path`. `Refresh`: set the zone folder at `path` to its zone's current paths and bring it to its zone
/// branch's tip. `Attempts`: make `n` attempt folders of the current request folder under `dir`. `Keep`: keep
/// attempt `k`. `Drop`: drop every attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderCommand {
    Zone { zone: String, path: PathBuf, start: Option<String> },
    Request { zone: String, request: String, path: PathBuf, paths: Vec<String> },
    Remove { path: PathBuf },
    Refresh { path: PathBuf },
    Attempts { n: u32, dir: PathBuf },
    Keep { k: u32 },
    Drop,
}
