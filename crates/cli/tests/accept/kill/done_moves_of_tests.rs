use super::{done_moves_of, OpLine};

fn op(op_id: &str, phase: &str, refname: &str) -> OpLine {
    OpLine {
        op_id: op_id.to_string(),
        phase: phase.to_string(),
        moves: vec![(refname.to_string(), "new".to_string())],
        ..OpLine::default()
    }
}

fn ids(lines: Vec<&OpLine>) -> Vec<&str> {
    lines.into_iter().map(|line| line.op_id.as_str()).collect()
}

#[test]
fn counts_done_moves_of_the_ref_from_the_killed_ops_intent_on() {
    let log = [
        op("old", "done", "refs/heads/zone/a"),
        op("k", "intent", "refs/heads/zone/a"),
        op("k", "failed", "refs/heads/zone/a"),
        op("n", "intent", "refs/heads/zone/a"),
        op("n", "done", "refs/heads/zone/a"),
        op("s", "done", "refs/mortar/snap/a/r1"),
    ];
    assert_eq!(ids(done_moves_of(&log, "refs/heads/zone/a", "k")), vec!["n"]);
}

#[test]
fn the_killed_ops_own_done_record_counts() {
    let log = [op("k", "intent", "refs/heads/integration"), op("k", "done", "refs/heads/integration")];
    assert_eq!(ids(done_moves_of(&log, "refs/heads/integration", "k")), vec!["k"]);
}

#[test]
fn with_no_record_of_the_op_the_whole_log_counts() {
    let log = [op("a", "done", "refs/heads/main"), op("b", "done", "refs/heads/main")];
    assert_eq!(ids(done_moves_of(&log, "refs/heads/main", "absent")), vec!["a", "b"]);
}
