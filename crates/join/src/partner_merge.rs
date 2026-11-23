//! PartnerMerge: what making a partner merge gave. A declaration only; tests/partner_base exercises it through
//! make_partner_merge and join_on_base.

use crate::request_base::RequestBase;
use std::path::PathBuf;

/// The merge of a zone tip with its partners' tips, or the partner whose merge conflicted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartnerMerge {
    /// The merge made (or the zone tip when no partner is left) and the partners kept, in the order given.
    Merged(RequestBase),
    /// Merging partner `zone` conflicted on `files`; no commit is answered, no ref moved, no state written.
    Conflicted { zone: String, files: Vec<PathBuf> },
}
