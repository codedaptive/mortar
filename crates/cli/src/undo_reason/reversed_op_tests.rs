//! reversed_op: reads (kind, op id) back from an undo's reason "undo of <kind> op <op_id>"; a reason in any other
//! form gives None.

use super::{reversed_op, undo_reason};

fn pair(kind: &str, op_id: &str) -> Option<(String, String)> {
    Some((kind.to_string(), op_id.to_string()))
}

#[test]
fn a_save_reason_gives_save_and_its_op() {
    assert_eq!(reversed_op("undo of save op op-7"), pair("save", "op-7"));
}

#[test]
fn a_join_rebase_reason_gives_join_rebase_and_its_op() {
    assert_eq!(reversed_op("undo of join-rebase op 1700000000-ab12"), pair("join-rebase", "1700000000-ab12"));
}

#[test]
fn undo_reasons_reason_reads_back_as_the_targets_kind_and_op() {
    for (kind, op_id) in [
        ("save", "op-1"),
        ("join-rebase", "1700000000-ab12"),
        ("undo", "op-99"),
        ("folder-move", "a-b-c-d"),
        ("restore", "x"),
    ] {
        let reason = undo_reason(kind, op_id);
        assert_eq!(reversed_op(&reason), pair(kind, op_id), "{reason}");
    }
}

#[test]
fn a_reason_not_in_that_form_gives_none() {
    for reason in [
        "",
        "fix the parser",
        "undo of save",
        "undo of save op",
        "undo of save op ",
        "undo of  op op-7",
        "redo of save op op-7",
        "Undo of save op op-7",
        "mortar restore op-1",
    ] {
        assert_eq!(reversed_op(reason), None, "{reason:?}");
    }
}
