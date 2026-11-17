use super::OpLine;

fn line(phase: &str, result: &str) -> OpLine {
    OpLine {
        op_id: "op-7".to_string(),
        kind: "land".to_string(),
        phase: phase.to_string(),
        result: result.to_string(),
        reason: "land C1 moves refs/heads/integration from a to b".to_string(),
        moves: Vec::new(),
    }
}

#[test]
fn an_intent_shows_no_result() {
    assert_eq!(line("intent", "").shown(), "op op-7 land intent: land C1 moves refs/heads/integration from a to b");
}

#[test]
fn a_closing_record_shows_its_result() {
    assert_eq!(
        line("failed", "not-moved").shown(),
        "op op-7 land failed (not-moved): land C1 moves refs/heads/integration from a to b"
    );
}
