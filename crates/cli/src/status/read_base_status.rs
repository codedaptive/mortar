//! read_base_status: a request folder's recorded base and partners, and which partners moved since.
//! No unit test: it runs git and reads the folder record; partner_moved_tests covers the decision and
//! with_base_status_tests the report. The integration tests
//! crates/cli/tests/status_need_runs/status_partners_tests.rs cover it through `mr status --json` in a request folder
//! cut with and without `--with`, before and after a partner zone moves.

use folders::plan::zone_branch;
use folders::RecordedBase;
use gitio::Git;

use super::{partner_moved, BaseStatus};
use crate::context::FolderContext;
use crate::request_base::read_request_base;

/// read_request_base on the folder, each partner's current tip of refs/heads/zone/<zone>, then partner_moved. A
/// record that cannot be read or is missing is read_request_base's text; a zone name or ref that cannot be read is
/// its error's text. Reads only: no lock, no ref move.
pub fn read_base_status(git: &Git, folder: &FolderContext) -> Result<BaseStatus, String> {
    let RecordedBase { base, partners } = read_request_base(folder)?;
    let current = partners
        .iter()
        .map(|partner| {
            let branch = zone_branch(&partner.zone).map_err(|error| error.reason)?;
            git.read_ref(&format!("refs/heads/{branch}")).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let moved = partner_moved(&partners, &current);
    Ok(BaseStatus { base, partners, moved })
}
