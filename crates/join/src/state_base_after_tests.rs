//! state_base_after: a state with a base answers that base, partners kept; one without answers its onto with no
//! partners.

use crate::join_state::JoinState;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use crate::state_base_after::state_base_after;
use gitio::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(base: Option<RequestBase>) -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid("1111111111111111111111111111111111111111"),
        onto: oid("2222222222222222222222222222222222222222"),
        replayed_tip: oid("3333333333333333333333333333333333333333"),
        commit: oid("4444444444444444444444444444444444444444"),
        tree: oid("5555555555555555555555555555555555555555"),
        remaining: Vec::new(),
        files: Vec::new(),
        all_files: Vec::new(),
        files_written: true,
        rebased_to: None,
        folder_tree: None,
        base,
        base_before: Some(RequestBase { base: oid("6666666666666666666666666666666666666666"), partners: vec![] }),
    }
}

#[test]
fn the_kept_base_else_onto_with_no_partners() {
    let merge = RequestBase {
        base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        partners: vec![Partner { zone: "docs".to_string(), tip: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb") }],
    };
    let rows = [
        (
            Some(merge),
            RequestBase {
                base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                partners: vec![Partner {
                    zone: "docs".to_string(),
                    tip: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                }],
            },
        ),
        (None, RequestBase { base: oid("2222222222222222222222222222222222222222"), partners: vec![] }),
    ];
    for (base, expected) in rows {
        assert_eq!(state_base_after(&state(base.clone())), expected, "{base:?}");
    }
}
