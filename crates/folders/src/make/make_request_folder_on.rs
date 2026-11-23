//! Makes a request's folder on a given base: its branch and a sparse worktree at that commit, the record holding
//! the base and the partner zones merged into it.
//! No unit test: it takes a lock, appends to the op log, runs git and writes files. other_base_refusal_tests and
//! repeat_decision_tests cover its decisions. tests/folders/partner_base.rs covers, in a throwaway repository: a
//! folder made on a merge commit with a partner, and on the zone tip with none, its branch and HEAD at the base and
//! its record holding base and partners; a repeat refills it, answers the same record and logs no new op; a repeat
//! with another base refuses with BranchExists and one with other partners with DifferentFolder, nothing changed;
//! a no-partner repeat at the zone tip on the partnered folder refuses with BranchExists; a no-partner folder
//! repeated at a moved zone tip with no partners refills and answers its record at the old base.

use gitio::Oid;

use super::make_or_refill::make_or_refill;
use super::other_base_refusal::other_base_refusal;
use super::read_record_and_partners::read_record_and_partners;
use super::with_folders_lock::with_folders_lock;
use super::{existing_folder, repeat_decision, FolderError, FolderSetup, Repeat};
use crate::plan::{request_branch, sparse_paths, zone_branch};
use crate::record::{FolderKind, FolderRecord, RecordedBase};

/// Makes the folder for `request` in `zone` at setup.path, holding `paths` (the work order's paths), on
/// base.base, recording base.partners (the partner zones merged into it, in order), and returns its record. The
/// names, each partner's zone name (zone_branch) and the cone are checked first, each refused as Plan. Under the
/// folders lock, keyed `folders:make:req/<zone>/<request>`: reads zone/<zone>'s tip (absent: ZoneBranchMissing);
/// zone/<zone> stays the record's parent while the record's base is base.base. When the folder at setup.path is
/// this request's (repeat_decision Same), its recorded base and partners are compared with `base` first
/// (other_base_refusal): another base refuses with BranchExists, the same base with other partners with
/// DifferentFolder, before anything is changed. The one exception: a call with no partners at the zone tip, on a
/// folder whose record holds no partners, skips the comparison and refills as make_request_folder does, answering
/// the folder's record whatever base it records. Then make_or_refill makes the branch and worktree at base.base
/// and writes the record with base.partners, or refills a folder already there and answers its record.
///
/// NOTE: make_request_folder refills a folder already there whatever its recorded base and partners; this form
/// refuses another base or other partners except in the exception above.
///
/// The caller runs oplog::recover first, as apply requires.
pub fn make_request_folder_on(
    setup: &FolderSetup,
    zone: &str,
    request: &str,
    paths: &[String],
    base: &RecordedBase,
) -> Result<FolderRecord, FolderError> {
    let zb = zone_branch(zone).map_err(FolderError::Plan)?;
    let rb = request_branch(zone, request).map_err(FolderError::Plan)?;
    let cone = sparse_paths(paths, &[]).map_err(FolderError::Plan)?;
    for partner in &base.partners {
        zone_branch(&partner.zone).map_err(FolderError::Plan)?;
    }
    let git = setup.git;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:make:{rb}"), setup.pids, || {
        let tip = git
            .read_ref(&format!("refs/heads/{zb}"))
            .map_err(FolderError::Git)?
            .ok_or_else(|| FolderError::ZoneBranchMissing { branch: zb.clone() })?;
        let wanted = FolderRecord {
            kind: FolderKind::Request,
            zone: zone.to_string(),
            request: Some(request.to_string()),
            base: base.base.clone(),
            work_order_paths: cone.clone(),
            needed_paths: Vec::new(),
            parent: Some(zb.clone()),
        };
        if let Some(refusal) = repeat_refusal(setup, &wanted, base, &tip, &rb)? {
            return Err(refusal);
        }
        make_or_refill(setup, &common, wanted, &base.partners, &rb, &base.base, &cone)
    })
}

/// other_base_refusal's answer for the folder at setup.path when repeat_decision finds it `wanted`; None when no
/// folder is there or another one is, which make_or_refill then makes or refuses. Also None when `base` asks for
/// no partners at the zone tip `tip` and the folder's record holds no partners: a folder make_request_folder could
/// have made, which is refilled as make_request_folder refills it.
fn repeat_refusal(
    setup: &FolderSetup,
    wanted: &FolderRecord,
    base: &RecordedBase,
    tip: &Oid,
    branch: &str,
) -> Result<Option<FolderError>, FolderError> {
    let Some(existing) = existing_folder(setup.git, &setup.path)? else {
        return Ok(None);
    };
    if repeat_decision(wanted, &existing) == Repeat::Different {
        return Ok(None);
    }
    let git_dir = setup.git.git_dir(&setup.path).map_err(FolderError::Git)?;
    let partners = read_record_and_partners(&git_dir)?.map(|(_, partners)| partners).unwrap_or_default();
    if base.partners.is_empty() && base.base == *tip && partners.is_empty() {
        return Ok(None);
    }
    Ok(other_base_refusal(base, &existing, &partners, branch, &setup.path))
}
