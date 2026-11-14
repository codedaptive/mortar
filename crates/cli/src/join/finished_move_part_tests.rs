//! finished_move_part: no move is no line and two Null fields; a move is its line and its two tips as text.

use gitio::Oid;

use super::{finished_move_part, PendingMove};
use crate::report::OutValue;

#[test]
fn no_move_is_no_line_and_null_tips() {
    assert_eq!(
        finished_move_part(None),
        (
            Vec::<String>::new(),
            vec![
                ("finished_move_from".to_string(), OutValue::Null),
                ("finished_move_to".to_string(), OutValue::Null),
            ]
        )
    );
}

#[test]
fn a_move_is_its_line_and_its_tips() {
    let moved = PendingMove {
        request_ref: "refs/heads/req/core/fix-1".to_string(),
        old: Oid::parse("1111111111111111111111111111111111111111").expect("a valid oid"),
        new: Oid::parse("2222222222222222222222222222222222222222").expect("a valid oid"),
        op_id: "op-rebase".to_string(),
        folder_tree: None,
    };
    assert_eq!(
        finished_move_part(Some(&moved)),
        (
            vec!["the folder stands at the last rebase's tip: 1111111111111111111111111111111111111111 -> \
                  2222222222222222222222222222222222222222"
                .to_string()],
            vec![
                (
                    "finished_move_from".to_string(),
                    OutValue::Text("1111111111111111111111111111111111111111".to_string())
                ),
                (
                    "finished_move_to".to_string(),
                    OutValue::Text("2222222222222222222222222222222222222222".to_string())
                ),
            ]
        )
    );
}
