//! read_install_facts: Reads everything install decides from: this binary, what stands at its paths, PATH, the
//! harness folders and the skill files. Reads files, never runs them, and writes nothing.
//! No sibling test: it touches the disk; the install ZITs in install/integration/ cover it (a fresh HOME, a repeat
//! install, a foreign `mortar`, `mr` or skill file, harness folders present or not, `--location local`).

use std::fs;
use std::path::Path;

use crate::path::{file_has_marker, is_this_build, MORTAR_MARKER};

use super::{
    content_hash, read_path_facts, skill_path, BinaryAt, InstallEnv, InstallFacts, InstallPaths, LinkAt, RecordItem,
    SkillAt, SkillLocation, HARNESSES,
};

/// The facts for `env` and `paths`, skill paths at `location`, carrying `record`. `this` is `env.exe` canonicalized
/// and `this_hash` content_hash of its bytes (None when either cannot be read). Mortar-ness is is_this_build against
/// `this` (false when `this` is None). A harness is detected when its folder in HOME is a directory.
pub fn read_install_facts(
    env: &InstallEnv,
    paths: &InstallPaths,
    location: SkillLocation,
    record: Vec<RecordItem>,
) -> InstallFacts {
    let this = env.exe.as_deref().and_then(|exe| fs::canonicalize(exe).ok());
    let bytes = this.as_deref().and_then(|this| fs::read(this).ok());
    let mortar = |candidate: &Path| {
        this.as_deref().is_some_and(|this| {
            is_this_build(candidate, this, &|p| fs::canonicalize(p).ok(), &|p| file_has_marker(p, &MORTAR_MARKER))
        })
    };
    let detected: Vec<_> = HARNESSES.iter().filter(|harness| env.home.join(harness.folder).is_dir()).collect();
    let skills = detected
        .iter()
        .map(|harness| {
            let path = skill_path(harness, location, &env.home, &env.project);
            let at = skill_at(&path);
            (*harness, path, at)
        })
        .collect();
    let path_facts = read_path_facts(env, paths);
    InstallFacts {
        binary_at: binary_at(&paths.mortar, this.as_deref(), bytes.as_deref(), &mortar),
        link_at: link_at(&paths.mr, &mortar),
        this_hash: bytes.as_deref().map(content_hash),
        this,
        first_mr: path_facts.first_mr,
        first_mortar: path_facts.first_mortar,
        first_mr_at: path_facts.first_mr_at,
        bin_at: path_facts.bin_at,
        exe_dir: env.exe.as_deref().and_then(Path::parent).map(Path::to_path_buf),
        detected,
        skills,
        record,
    }
}

/// What stands at `at` (`~/.local/bin/mortar`): nothing; this binary (its canonical path is `this`, or its bytes are
/// `bytes`); another Mortar build; or anything else.
fn binary_at(at: &Path, this: Option<&Path>, bytes: Option<&[u8]>, mortar: &dyn Fn(&Path) -> bool) -> BinaryAt {
    if fs::symlink_metadata(at).is_err() {
        return BinaryAt::Absent;
    }
    let same_path = this.is_some_and(|this| fs::canonicalize(at).is_ok_and(|at| at == this));
    let same_bytes = bytes.is_some_and(|bytes| fs::read(at).is_ok_and(|found| found == bytes));
    if same_path || same_bytes {
        return BinaryAt::Same;
    }
    if mortar(at) {
        BinaryAt::Mortar
    } else {
        BinaryAt::Foreign
    }
}

/// What stands at `at` (`~/.local/bin/mr`): nothing; a symlink with its target as read and whether it resolves to
/// Mortar; a file that is Mortar; or anything else.
fn link_at(at: &Path, mortar: &dyn Fn(&Path) -> bool) -> LinkAt {
    let Ok(meta) = fs::symlink_metadata(at) else {
        return LinkAt::Absent;
    };
    match (meta.file_type().is_symlink(), fs::read_link(at), mortar(at)) {
        (true, Ok(target), is) => LinkAt::Link { target, mortar: is },
        (false, _, true) => LinkAt::Mortar,
        _ => LinkAt::Foreign,
    }
}

/// What stands at a skill path: nothing, a regular file with content_hash of its bytes, or anything else.
fn skill_at(path: &Path) -> SkillAt {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return SkillAt::Absent;
    };
    match (meta.is_file(), fs::read(path)) {
        (true, Ok(bytes)) => SkillAt::File { hash: content_hash(&bytes) },
        _ => SkillAt::NotFile,
    }
}
