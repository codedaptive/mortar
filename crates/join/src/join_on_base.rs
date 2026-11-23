//! join_on_base: joins a request made on a partner merge to its zone. It takes no lock: the caller holds
//! join-<zone> and has run recover.
//! No unit test: it runs git, applies ops and, through pause, writes real files; decide_base_join_tests covers its
//! decision, zone_request_op_tests and rebase_op_tests its ops, after_replay_tests the paused state it keeps.
//! tests/partner_base covers it in a throwaway repository: an unchanged base joins the request's own commits to the
//! zone by one op and a repeat answers Plain(Joined); a moved partner rebases the request onto a new partner merge,
//! or is refused as PartnersConflicted with nothing moved; a replay conflict pauses keeping the new base, which
//! resolve_on_base hands back; a replay conflict onto the unchanged zone pauses keeping the zone tip with no
//! partners, which resolve_on_base hands back; no partners answers join's own answer. tests/partner_base's
//! recorded_base.rs covers the base before and after each op records, plain, remerged and onto the unchanged zone.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::base_join_decision::BaseJoinDecision;
use crate::base_join_read::BaseJoinRead;
use crate::decide_base_join::decide_base_join;
use crate::pause::pause;
use crate::join_recording::join_recording;
use crate::rebased::rebased;
use crate::zone_joined::zone_joined;
use crate::{make_partner_merge, replay, BaseJoinAnswer, JoinAnswer, JoinRunError, Partner, PartnerMerge, RequestBase};
use gitio::{Git, Oid};
use std::path::Path;

/// Joins `request_ref` (a full ref name), made on `base`, to zone `zone` (ref `refs/heads/zone/<zone>`). With no
/// partners it is join recording `base` as the base before (join_recording), its answer Plain. Otherwise it reads
/// the zone tip, the request tip and each partner's current tip at `refs/heads/zone/<partner>` (a missing one
/// ZoneMissing naming its ref), and decides: a request tip already in the zone answers Plain(Joined) and moves
/// nothing. Zone and partners where the base had them: the request's commits after the base are replayed onto the
/// zone tip (never a fast-forward through the merge) and, clean, one op moves the zone ref and the request ref to
/// the replayed tip: ZoneJoined. The zone move is a fast-forward by construction; when the op moves the zone ref (the
/// replay left a commit of the request's own; the zone move is left out when `to` is the zone tip), its old value pins
/// the zone ref at the tip read, so a zone moved since fails the op (OpFailed) with nothing moved (zone_joined).
/// Anything moved: the partner merge is made again from the zone tip and the partners' current tips (a conflict is
/// PartnersConflicted, nothing moved, no state) and the request's commits after the base are replayed onto it;
/// clean, one op moves the request ref: Rebased with the new base. A conflicted replay pauses as join does and
/// answers Plain(Conflicted); the state keeps the base the request sits on once resolved: the zone tip with no
/// partners when replayed onto the unchanged zone, the new merge and its kept partners after a remerge. The
/// completing resolve_on_base hands that base back, and the state keeps `base` as its base before. Each op that
/// moves the request ref records base_values(`base`, the base after: the zone tip with no partners, or the new merge
/// and its partners). Ops name `actor`; a done record already holding an op's key is OpRepeated.
pub fn join_on_base(
    git: &Git,
    folder: &Path,
    zone: &str,
    request_ref: &str,
    base: &RequestBase,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    if base.partners.is_empty() {
        return join_recording(git, folder, zone, request_ref, actor, Some(base)).map(BaseJoinAnswer::Plain);
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
            match replayed(git, &from_base, request_ref, &request_tip, &onto, &on_zone, base)? {
                AfterReplayStep::Rebase { from, to } => {
                    zone_joined(git, zone_ref, onto, request_ref, from, to, base, actor)
                }
                AfterReplayStep::Pause(state) => pause(git, folder, state).map(BaseJoinAnswer::Plain),
            }
        }
        BaseJoinDecision::Remerge => {
            match make_partner_merge(git, &read.zone_tip, &current).map_err(JoinRunError::Replay)? {
                PartnerMerge::Conflicted { zone, files } => Ok(BaseJoinAnswer::PartnersConflicted { zone, files }),
                PartnerMerge::Merged(new) => {
                    match replayed(git, &base.base, request_ref, &request_tip, &new.base, &new, base)? {
                        AfterReplayStep::Rebase { from, to } => rebased(git, request_ref, from, to, base, new, actor),
                        AfterReplayStep::Pause(state) => pause(git, folder, state).map(BaseJoinAnswer::Plain),
                    }
                }
            }
        }
    }
}

/// Replays the request's commits after `from_base` onto `onto` and takes the step after it, a pause keeping `keep`
/// as its base and `before` as its base before.
fn replayed(
    git: &Git,
    from_base: &Oid,
    request_ref: &str,
    request_tip: &Oid,
    onto: &Oid,
    keep: &RequestBase,
    before: &RequestBase,
) -> Result<AfterReplayStep, JoinRunError> {
    let result = replay(git, from_base, request_ref, onto).map_err(JoinRunError::Replay)?;
    Ok(after_replay(request_ref, request_tip, onto, &[], Some(keep), Some(before), result))
}
