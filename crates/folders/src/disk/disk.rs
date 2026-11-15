//! The disk report: each Mortar folder, the object store, the refs, and what a sweep would free.
//! No unit test: it runs git and reads the file system (S-5); sweep_figure_tests covers the figure,
//! refs_by_namespace_tests the ref counts, idle_folders_tests the idle choice, and ref_sweep_bytes' header the ref
//! choice. Nagatha's integration group is to cover, in a throwaway repository: disk on a repository with a zone
//! folder and a request folder lists both with their files and build bytes, counts the object store once, and counts
//! the refs by namespace; the main checkout is not listed; the sweep figure is the build bytes of the idle folders
//! plus disk_usage_only over the refs sweep_refs would delete; a refused extra ref refuses the report with
//! ExtraRefRefused. tests/folders/sweeps_replayed.rs covers the sweep figure's count of replay-record refs
//! (refs/mortar/replayed/<zone>/<source>) of zones not in the zone list (S-121 (2)).

use std::path::Path;

use gitio::Git;
use oplog::{unix_now, ProtectedRef};

use super::{
    measure_folder, measure_tree, mortar_folders, ref_sweep_bytes, refs_by_namespace, sweep_figure, DiskReport,
    FolderDisk,
};
use crate::make::FolderError;
use crate::sweep::{idle_folders, FolderAge};

/// Measures each Mortar folder by measure_folder, the measure sweep_build judges idleness by: files_bytes is the
/// folder without `.git` and DEFAULT_BUILD_DIR, build_bytes the build dir alone, newest the later of the two.
/// store_bytes is the common dir's `objects`, counted once because every folder shares it. refs counts every ref by
/// namespace. The sweep figure is sweep_figure over the build bytes of the folders idle at least `idle_days` days
/// now and ref_sweep_bytes over the refs sweep_refs would delete now; `zones`, `extra_refs` and `protected` are
/// sweep_refs' own, so the figure covers exactly what that sweep would delete, an upper bound (S-75). A refused
/// extra ref is ExtraRefRefused (S-53, S-54). `idle_days` is a parameter because the figure depends on it and
/// SPEC-folders' `disk()` leaves it open. Takes no lock (S-16).
pub fn disk(
    git: &Git,
    idle_days: u64,
    zones: &[String],
    extra_refs: &[String],
    protected: &[ProtectedRef],
) -> Result<DiskReport, FolderError> {
    let mut folders = Vec::new();
    for (path, record) in mortar_folders(git)? {
        let measure = measure_folder(&path)?;
        folders.push(FolderDisk {
            path,
            record,
            files_bytes: measure.files,
            build_bytes: measure.build,
            newest: measure.newest,
        });
    }
    let common = git.common_dir().map_err(FolderError::Git)?;
    let store_bytes = measure_tree(&common.join(Path::new("objects")), &[])?.bytes;
    let refs = git.list_refs("").map_err(FolderError::Git)?;
    let ages: Vec<FolderAge> =
        folders.iter().map(|f| FolderAge { path: f.path.clone(), newest: f.newest }).collect();
    let idle = idle_folders(&ages, unix_now(), idle_days);
    let (swept_refs, refs_bytes) = ref_sweep_bytes(git, &refs, zones, extra_refs, protected)?;
    let sweep = sweep_figure(&folders, &idle, idle_days, swept_refs, refs_bytes);
    Ok(DiskReport {
        folders,
        store_bytes,
        refs: refs_by_namespace(&refs),
        sweep,
    })
}
