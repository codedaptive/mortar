//! cut_on_partners: the request folder of `mr folder request ... --with <zone>...`, made on a merge of the zone's tip
//! with each partner zone's tip, for run_folder_request.
//! No sibling test: it reads refs and merges with git through join and folders; partner_conflict_report_tests covers
//! its refusal. The integration test crates/cli/tests/folder_runs/partner_cut_tests.rs covers it through the binary:
//! the folder cut on the merge of zone and partner (partner_cut_and_repeat), a partner the zone already holds dropped
//! (partner_cut_reports_partner_already_in_zone) and a conflicting merge refused with nothing made
//! (partner_cut_conflict_makes_nothing).

use ::join::{make_partner_merge, Partner, PartnerMerge};
use folders::plan::zone_branch;
use folders::{make_request_folder_on, FolderError, FolderRecord, FolderSetup};
use gitio::Oid;

use super::{folder_error_report, partner_conflict_report};
use crate::report::{failure_report, Report};
use crate::request_base::recorded_base_of;

const VERB: &str = "folder";

/// Reads refs/heads/zone/<zone> and each partner's, in order (a missing one is ZoneBranchMissing naming
/// `zone/<name>`, as folders names it), merges them with join's make_partner_merge, and makes the folder on the
/// merge with make_request_folder_on, recording the partners the merge kept (join drops a partner the zone already
/// holds). A conflict is partner_conflict_report's refusal and join's error failure_report, both before anything is
/// made.
pub(crate) fn cut_on_partners(
    setup: &FolderSetup,
    zone: &str,
    request: &str,
    paths: &[String],
    partners: &[String],
) -> Result<FolderRecord, Report> {
    let refused = |error: FolderError| folder_error_report("request", &error);
    let tip_of = |name: &str| -> Result<Oid, FolderError> {
        let branch = zone_branch(name).map_err(FolderError::Plan)?;
        let tip = setup.git.read_ref(&format!("refs/heads/{branch}")).map_err(FolderError::Git)?;
        tip.ok_or(FolderError::ZoneBranchMissing { branch })
    };
    let zone_tip = tip_of(zone).map_err(refused)?;
    let partners = partners
        .iter()
        .map(|name| Ok(Partner { zone: name.clone(), tip: tip_of(name)? }))
        .collect::<Result<Vec<_>, FolderError>>()
        .map_err(refused)?;
    match make_partner_merge(setup.git, &zone_tip, &partners) {
        Err(error) => Err(failure_report(VERB, &format!("mr folder request: {error}"))),
        Ok(PartnerMerge::Conflicted { zone: partner, files }) => Err(partner_conflict_report(zone, &partner, &files)),
        Ok(PartnerMerge::Merged(base)) => {
            make_request_folder_on(setup, zone, request, paths, &recorded_base_of(&base)).map_err(refused)
        }
    }
}
