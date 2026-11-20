use super::{drive_passed, Block, Verdict};

fn block(verdict: Verdict) -> Block {
    Block { number: 0, title: "t", verdict, lines: vec![] }
}

#[test]
fn pass_and_not_run_pass_the_drive() {
    assert!(drive_passed(&[block(Verdict::Pass), block(Verdict::NotRun("set X".to_string()))]));
}

#[test]
fn one_failed_block_fails_the_drive() {
    assert!(!drive_passed(&[block(Verdict::Pass), block(Verdict::Fail), block(Verdict::Pass)]));
}
