//! Public Land batch APIs -> real Gitio object IDs and Oplog JSON, with independent literal state layouts.
use crate::{batch_fixture::{Batch, ACTOR, IDS, INTEGRATION}, owed_ops::{locked, refs}};
use land::{BatchBackOutAnswer, BatchLandAnswer, LandError, LandingStateError};
use oplog::ChangeId;
use std::fs;

fn back_out(b: &Batch, count: usize) {
    assert_eq!(locked(&b.f, || land::back_out_batch(&b.f.git, &refs(), ACTOR)), BatchBackOutAnswer::BackedOut {
        change_id: ChangeId::parse(IDS[count - 1]).unwrap(),
        change_ids: IDS[..count].iter().map(|id| ChangeId::parse(id).unwrap()).collect(),
    });
    assert_eq!(b.f.git.read_ref(INTEGRATION).unwrap(), Some(b.origin.clone())); assert!(!b.path.exists());
}

#[test]
fn rit_land_batch_encodes_order_and_reachable_escapes_then_back_out_removes_state() {
    let b = Batch::new("a\"b\\c", r#"a\"b\\c"#); b.land(0); b.land(1);
    assert_eq!(fs::read(&b.path).unwrap(), b.text(2, None).as_bytes());
    assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), Some(b.members(2)));
    back_out(&b, 2);
}

#[test]
fn rit_handwritten_batch_decodes_member_identity_and_back_out_order_with_or_without_pending() {
    for pending in [false, true] {
        let b = Batch::new("second", "second"); let candidate = b.pending(1);
        let text = b.text(2, pending.then_some(&candidate)); b.seed(&text, &b.landed[1]);
        let before = b.observe();
        assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), Some(b.members(2)));
        b.land(0); // A repeat of the first member must not answer the newest one's integration.
        assert_eq!(b.observe(), before, "read/repeat must not write: pending={pending}");
        back_out(&b, 2);
    }
}

#[test]
fn rit_land_batch_appends_to_handwritten_batch_with_or_without_pending() {
    for pending in [false, true] {
        let b = Batch::new("second", "second"); let candidate = b.pending(1);
        b.seed(&b.text(2, pending.then_some(&candidate)), &b.landed[1]); b.land(2);
        assert_eq!(fs::read(&b.path).unwrap(), b.text(3, None).as_bytes());
        assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), Some(b.members(3)));
        assert_eq!(b.f.git.commit_info(&b.landed[2]).unwrap().parents, vec![b.landed[1].clone(), b.squashes[2].commit.clone()]);
    }
}

#[test]
fn rit_corrupt_batch_files_are_refused_without_ref_log_or_state_writes() {
    let b = Batch::new("second", "second");
    let empty = format!("{{\"origin_at_start\":\"{}\",\"landings\":[]}}\n", b.origin);
    let missing = b.text(2, None).replacen(",\"zone\":\"core\"", "", 1);
    for (text, reason) in [
        (empty, "expected a landing in the landings array at byte 74: it is empty"),
        (missing, "expected ,\"zone\": at byte 121"),
        (format!("{}x", b.text(2, None)), "expected the closing brace and the end of the text at byte 420"),
    ] {
        b.seed(&text, &b.landed[1]); let before = b.observe();
        let answer = locked(&b.f, || Ok::<_, LandError>(land::land_batch(&b.f.git, &refs(), &b.squashes[2], ACTOR)));
        match answer {
            Err(LandError::State(LandingStateError::Corrupt { path, reason: actual })) => {
                assert_eq!(path, b.path); assert_eq!(actual, reason);
            }
            other => panic!("expected Corrupt for {text:?}: {other:?}"),
        }
        assert_eq!(b.observe(), before); assert!(!b.path.with_file_name("landing.json.tmp").exists());
    }
}

#[test]
fn rit_land_batch_repeats_legacy_state_without_writes_then_appends_in_batch_form() {
    let b = Batch::new("second", "second"); b.seed(&b.legacy(None), &b.landed[0]); let before = b.observe();
    b.land(0); assert_eq!(b.observe(), before); b.land(1);
    assert_eq!(fs::read(&b.path).unwrap(), b.text(2, None).as_bytes());
}

#[test]
fn rit_read_landing_batch_joins_both_file_layouts_without_resolving_or_changing_members() {
    let b = Batch::new("second", "second"); let before = b.observe();
    assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), None); assert_eq!(b.observe(), before);
    for count in [1, 2] {
        let candidate = b.pending(count - 1);
        for pending in [None, Some(&candidate)] {
            let text = if count == 1 { b.legacy(pending) } else { b.text(count, pending) };
            for tip in [&b.landed[count - 1], &candidate] {
                b.seed(&text, tip); let before = b.observe();
                assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), Some(b.members(count)));
                assert_eq!(b.observe(), before);
            }
        }
    }
}

#[test]
fn rit_pending_in_both_layouts_is_folded_only_when_integration_reached_it() {
    let b = Batch::new("second", "second");
    for count in [1, 2] {
        let candidate = b.pending(count - 1);
        let text = if count == 1 { b.legacy(Some(&candidate)) } else { b.text(count, Some(&candidate)) };
        for tip in [&b.landed[count - 1], &candidate] {
            b.seed(&text, tip); let before = b.observe();
            let status = land::read_landing_status(&b.f.git, &refs()).unwrap().unwrap();
            assert_eq!(status.integration, *tip); assert_eq!(status.origin_at_start, b.origin);
            assert_eq!(status.change_id.as_str(), IDS[count - 1]); assert_eq!(status.squash, b.squashes[count - 1].commit);
            assert_eq!(b.observe(), before);
            assert_eq!(locked(&b.f, || land::land_batch(&b.f.git, &refs(), &b.squashes[count - 1], ACTOR)),
                BatchLandAnswer::Landed { integration: tip.clone() });
            assert_eq!(b.observe(), before);
        }
    }
}
