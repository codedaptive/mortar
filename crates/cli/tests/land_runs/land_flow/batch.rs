//! Batch landing verbs through the CLI in isolated repositories.
use super::fixture::Landing;

#[test]
fn batch_two_zones_land_in_order() {
    let s = Landing::without_remote(false);
    let peer = s.peer_squash();
    s.land();
    let r = s.run("land", Some(peer.commit.as_str()), 0, "landed");
    let (core, peer) = (s.squash.change_id.as_str(), peer.change_id.as_str());
    assert_eq!(r.strings("lines")[1], format!("landed 2 of the batch, in landing order: {core}, {peer}"));
    assert_eq!(r.strings("batch"), &[core, peer]);
}

#[test]
fn batch_second_landing_merges_disjoint_edits_in_the_folder() {
    let (s, peer) = Landing::disjoint_squashes();
    s.land();
    let r = s.run("land", Some(peer.commit.as_str()), 0, "landed");
    let tip = s.tip();
    assert_eq!(r.text("folder"), s.folder.to_str().unwrap());
    s.assert_clean_at(&tip, b"core\ntwo\nthree\nfour\nfive\nsix\npeer\n");
}

#[test]
fn batch_repeat_of_earlier_member_reports_the_real_tip_and_moves_nothing() {
    let s = Landing::without_remote(true);
    let peer = s.peer_squash();
    s.land();
    s.run("land", Some(peer.commit.as_str()), 0, "landed");
    let tip = s.tip();
    let before = s.observe();
    let r = s.land();
    assert_eq!(s.tip(), tip);
    assert_eq!(s.observe(), before);
    assert_eq!(r.text("integration"), tip.as_str());
    assert_eq!(r.strings("lines")[0], format!("landed change {} (commit {}): integration is at {tip}", s.squash.change_id, s.squash.commit));
    assert_eq!(r.strings("lines")[2], format!("moved the integration folder {} to {tip}", s.folder.display()));
    assert_eq!(r.text("folder"), s.folder.to_str().unwrap());
}

#[test]
fn batch_one_repeat_keeps_folder_line_without_another_op() {
    let s = Landing::without_remote(true);
    s.land();
    let records = s.f.records();
    let r = s.land();
    let id = s.squash.change_id.as_str();
    let commit = s.squash.commit.as_str();
    assert_eq!(r.strings("lines"), &[
        format!("landed change {id} (commit {commit}): integration is at {commit}"),
        format!("moved the integration folder {} to {commit}", s.folder.display()),
    ]);
    assert_eq!(r.text("folder"), s.folder.to_str().unwrap());
    assert_eq!(s.f.records(), records);
}

#[test]
fn batch_refuses_a_second_squash_of_the_same_zone_without_moving_state() {
    let s = Landing::without_remote(false);
    s.land();
    let further = s.further_core_squash();
    let before = s.observe();
    let r = s.run("land", Some(further.commit.as_str()), 3, "zone-in-batch");
    let member = s.squash.change_id.as_str();
    let id = further.change_id.as_str();
    let commit = further.commit.as_str();
    assert_eq!(r.text("zone"), "core");
    assert_eq!(r.text("member"), member);
    assert_eq!(r.text("change_id"), id);
    assert_eq!(r.text("commit"), commit);
    assert_eq!(r.strings("lines"), &[
        format!("zone core is already in the batch as change {member}; change {id} (commit {commit}) was not landed"),
        "integration and the batch were not moved".to_string(),
    ]);
    assert_eq!(r.text("owner"), format!("change {member}'s landing"));
    assert_eq!(r.text("next"), format!("publish or back out the batch first (mr publish --actor <name>, or mr back-out --actor <name>), then mr land {commit} --actor <name>"));
    assert_eq!(s.observe(), before);
}

#[test]
fn batch_back_out_names_both_members_in_landing_order() {
    let s = Landing::without_remote(false);
    let peer = s.peer_squash();
    s.kit_land();
    s.kit_append(&peer);
    let r = s.run("back-out", None, 0, "backed-out");
    let (core, peer) = (s.squash.change_id.as_str(), peer.change_id.as_str());
    assert_eq!(r.strings("lines")[0], format!("backed out changes {core}, {peer} (in landing order): integration is back at {}", s.f.base));
    assert_eq!(r.strings("backed_out"), &[core, peer]);
    assert_eq!(r.text("change_id"), peer);
    assert_eq!(s.tip(), s.f.base);
}

#[test]
fn batch_publish_names_both_zones_in_landing_order() {
    let s = Landing::new(false);
    let peer = s.peer_squash();
    s.kit_land();
    s.kit_append(&peer);
    let r = s.run("publish", None, 0, "published");
    assert_eq!(r.strings("lines")[1], "the landing state and the squash refs of zones core, peer are removed");
    assert_eq!(r.text("next"), "mr after-publish core --actor <name>, then mr after-publish peer --actor <name>");
}

#[test]
fn batch_one_keeps_text_and_adds_single_member_fields() {
    let s = Landing::new(false);
    let id = s.squash.change_id.as_str();
    let commit = s.squash.commit.as_str();
    let out = s.output("land", Some(commit));
    let r = crate::json::check(out, "land", 0, "landed");
    assert_eq!(r.strings("lines"), &[format!("landed change {id} (commit {commit}): integration is at {commit}"), "no integration folder to move".to_string()]);
    assert_eq!(r.strings("batch"), &[id]);
    let r = s.run("back-out", None, 0, "backed-out");
    assert_eq!(r.strings("lines"), &[format!("backed out change {id}: integration is back at {}", s.f.base), "the squash and its ref are kept, with the same change ID".to_string(), "no integration folder to move".to_string()]);
    assert_eq!(r.strings("backed_out"), &[id]);
    s.land();
    let r = s.run("publish", None, 0, "published");
    assert_eq!(r.strings("lines"), &[format!("published {commit}: origin's main and the local main are at it"), "the landing state and the zone's squash ref are removed".to_string()]);
    assert_eq!(r.text("next"), "mr after-publish core --actor <name>");
}
