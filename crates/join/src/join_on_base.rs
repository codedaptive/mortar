//! join_on_base: joins a request made on a partner merge to its zone. It takes no lock: the caller holds
//! join-<zone> and has run recover.
//! No unit test: it runs git, applies ops and, through pause, writes real files; decide_base_join_tests covers its
//! decision, zone_request_op_tests and rebase_op_tests its ops, after_replay_tests the paused state it keeps.
//! tests/partner_base covers it in a throwaway repository: an unchanged base joins the request's own commits to the
//! zone by one op and a repeat answers Plain(Joined); a moved partner rebases the request onto a new partner merge,
//! or is refused as PartnersConflicted with nothing moved; a replay conflict pauses keeping the new base, which
//! resolve_on_base hands back; a replay conflict onto the unchanged zone pauses keeping the zone tip with no
//! partners, which resolve_on_base hands back; no partners answers join's own answer.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::base_join_decision::BaseJoinDecision;
use crate::base_join_read::BaseJoinRead;
use crate::decide_base_join::decide_base_join;
use crate::op_applied::op_applied;
use crate::pause::pause;
use crate::rebase_op::rebase_op;
use crate::zone_request_op::zone_request_op;
use crate::{
    join, make_partner_merge, replay, BaseJoinAnswer, JoinAnswer, JoinRunError, Partner, PartnerMerge, RequestBase,
};
use gitio::{Git, Oid};
use oplog::{apply, apply_protected, new_op_id, unix_now, ProtectedRef};
use std::path::Path;

/// Joins `request_ref` (a full ref name), made on `base`, to zone `zone` (ref `refs/heads/zone/<zone>`). With no
/// partners it is join, its answer Plain. Otherwise it reads the zone tip, the request tip and each partner's current
/// tip at `refs/heads/zone/<partner>` (a missing one ZoneMissing naming its ref), and decides: a request tip already
/// in the zone answers Plain(Joined) and moves nothing. Zone and partners where the base had them: the request's
/// commits after the base are replayed onto the zone tip (never a fast-forward through the merge) and, clean, one
/// op moves the zone ref (protected, so only forward) and the request ref to the replayed tip: ZoneJoined.
/// Anything moved: the partner merge is made again from the zone tip and the partners' current tips (a conflict is
/// PartnersConflicted, nothing moved, no state) and the request's commits after the base are replayed onto it;
/// clean, one op moves the request ref: Rebased with the new base. A conflicted replay pauses as join does and
/// answers Plain(Conflicted); the state keeps the base the request sits on once resolved: the zone tip with no
/// partners when replayed onto the unchanged zone, the new merge and its kept partners after a remerge. The
/// completing resolve_on_base hands that base back. Ops name `actor`; a done record already holding an op's key is
/// OpRepeated.
pub fn join_on_base(
    git: &Git,
    folder: &Path,
    zone: &str,
    request_ref: &str,
    base: &RequestBase,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    if base.partners.is_empty() {
        return join(git, folder, zone, request_ref, actor).map(BaseJoinAnswer::Plain);
    }
    let zone_ref = format!("refs/heads/zone/{zone}");
    let zone_tip = git
        .read_ref(&zone_ref)
        .map_err(JoinRunError::Git)?
        .ok_or_else(|| JoinRunError::ZoneMissing(zone_ref.clone()))?;
    let request_tip = git
        .read_ref(request_ref)
        .map_err(JoinRunError::Git)?
        .ok_or_else(|| JoinRunError::RequestMissing(request_ref.to_string()))?;
    let mut current = Vec::new();
    for partner in &base.partners {
        let partner_ref = format!("refs/heads/zone/{}", partner.zone);
        let tip = git.read_ref(&partner_ref).map_err(JoinRunError::Git)?.ok_or(JoinRunError::ZoneMissing(partner_ref))?;
        current.push(Partner { zone: partner.zone.clone(), tip });
    }
    let read = BaseJoinRead {
        request_in_zone: git.is_ancestor(&request_tip, &zone_tip).map_err(JoinRunError::Git)?,
        zone_tip,
        base: base.base.clone(),
        base_parents: git.commit_info(&base.base).map_err(JoinRunError::Git)?.parents,
        partner_tips: current.iter().map(|partner| partner.tip.clone()).collect(),
    };
    match decide_base_join(&read) {
        BaseJoinDecision::Joined => Ok(BaseJoinAnswer::Plain(JoinAnswer::Joined { zone_ref, request_tip })),
        BaseJoinDecision::ReplayOntoZone { base: from_base, onto } => {
            let on_zone = RequestBase { base: onto.clone(), partners: vec![] };
            match replayed(git, &from_base, request_ref, &request_tip, &onto, &on_zone)? {
                AfterReplayStep::Rebase { from, to } => zone_joined(git, zone_ref, onto, request_ref, from, to, actor),
                AfterReplayStep::Pause(state) => pause(git, folder, state).map(BaseJoinAnswer::Plain),
            }
        }
        BaseJoinDecision::Remerge => {
            match make_partner_merge(git, &read.zone_tip, &current).map_err(JoinRunError::Replay)? {
                PartnerMerge::Conflicted { zone, files } => Ok(BaseJoinAnswer::PartnersConflicted { zone, files }),
                PartnerMerge::Merged(new) => {
                    match replayed(git, &base.base, request_ref, &request_tip, &new.base, &new)? {
                        AfterReplayStep::Rebase { from, to } => rebased(git, request_ref, from, to, new, actor),
                        AfterReplayStep::Pause(state) => pause(git, folder, state).map(BaseJoinAnswer::Plain),
                    }
                }
            }
        }
    }
}

/// Replays the request's commits after `from_base` onto `onto` and takes the step after it, a pause keeping `keep`.
fn replayed(
    git: &Git,
    from_base: &Oid,
    request_ref: &str,
    request_tip: &Oid,
    onto: &Oid,
    keep: &RequestBase,
) -> Result<AfterReplayStep, JoinRunError> {
    let result = replay(git, from_base, request_ref, onto).map_err(JoinRunError::Replay)?;
    Ok(after_replay(request_ref, request_tip, onto, &[], Some(keep), result))
}

/// One op, applied with the zone ref protected by "join" so the zone moves only forward, moving the zone ref from
/// `zone_tip` and the request ref from `from`, both to `to`.
fn zone_joined(
    git: &Git,
    zone_ref: String,
    zone_tip: Oid,
    request_ref: &str,
    from: Oid,
    to: Oid,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = zone_request_op(&zone_ref, &zone_tip, request_ref, &from, &to, new_op_id(), actor, unix_now());
    let protected = [ProtectedRef { refname: zone_ref.clone(), by: "join".to_string() }];
    op_applied(apply_protected(git, &common, &op, &protected))?;
    Ok(BaseJoinAnswer::ZoneJoined {
        zone_ref,
        request_ref: request_ref.to_string(),
        from: zone_tip,
        request_from: from,
        to,
    })
}

/// One op moving the request ref from `from` to `to`, replayed onto the new merge `new.base`.
fn rebased(
    git: &Git,
    request_ref: &str,
    from: Oid,
    to: Oid,
    new: RequestBase,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = rebase_op(request_ref, &from, &to, &new.base, new_op_id(), actor, unix_now());
    op_applied(apply(git, &common, &op))?;
    Ok(BaseJoinAnswer::Rebased { request_ref: request_ref.to_string(), from, to, base: new })
}
