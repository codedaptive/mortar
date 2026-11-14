//! choose_change_id: whether a squash keeps the change ID of the squash already at its ref (S-57).

use crate::squash::PriorSquash;
use oplog::ChangeId;

/// The prior squash's ID when there is a prior, it is not yet in origin (so this squash is the same change,
/// edited or remade) and it carries an ID. None: the caller draws a new one, because the prior landed (a new
/// change follows it) or there is no ID to keep.
pub(crate) fn choose_change_id(prior: Option<&PriorSquash>) -> Option<ChangeId> {
    prior.filter(|prior| !prior.in_origin).and_then(|prior| prior.change_id.clone())
}
