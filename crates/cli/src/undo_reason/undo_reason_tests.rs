//! undo_reason: the reason an undo op records, "undo of <kind> op <op_id>", which reversed_op reads back.

use super::{reversed_op, undo_reason};

#[test]
fn a_save_op_is_named_by_kind_and_id() {
    assert_eq!(undo_reason("save", "op-7"), "undo of save op op-7");
}

#[test]
fn a_join_rebase_op_is_named_by_kind_and_id() {
    assert_eq!(undo_reason("join-rebase", "1700000000-ab12"), "undo of join-rebase op 1700000000-ab12");
}

#[test]
fn an_undo_op_is_named_by_kind_and_id() {
    assert_eq!(undo_reason("undo", "op-99"), "undo of undo op op-99");
}

#[test]
fn the_reason_reads_back_as_its_kind_and_op() {
    for (kind, op_id) in [("save", "op-1"), ("join-rebase", "1700000000-ab12"), ("folder-move", "a-b-c-d")] {
        assert_eq!(reversed_op(&undo_reason(kind, op_id)), Some((kind.to_string(), op_id.to_string())));
    }
}
