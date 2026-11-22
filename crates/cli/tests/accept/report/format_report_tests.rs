use super::{format_report, Block, Line, Verdict};

fn check(ok: bool, did: &str, saw: &str) -> Line {
    Line::Check { ok, did: did.to_string(), saw: saw.to_string() }
}

#[test]
fn blocks_lines_and_a_passing_result() {
    let blocks = [
        Block {
            number: 0,
            title: "the shell",
            verdict: Verdict::Pass,
            lines: vec![check(true, "ran init", "exit 0"), Line::Log("op 7 join done".to_string())],
        },
        Block {
            number: 4,
            title: "the agent test",
            verdict: Verdict::NotRun("set ACCEPT_AGENT to an agent".to_string()),
            lines: vec![],
        },
    ];
    let expected = "mortar acceptance report\n\
                    commit 0123456789abcdef0123456789abcdef01234567\n\
                    \n\
                    Step 0 the shell: PASS\n  \
                    ok    ran init: exit 0\n  \
                    log   op 7 join done\n\
                    \n\
                    Step 4 the agent test: NOT RUN (set ACCEPT_AGENT to an agent)\n\
                    \n\
                    Result: PASS\n";
    assert_eq!(format_report("commit 0123456789abcdef0123456789abcdef01234567", &blocks), expected);
}

#[test]
fn a_failed_block_fails_the_result_and_extra_lines_are_indented() {
    let blocks = [Block {
        number: 1,
        title: "the flow",
        verdict: Verdict::Fail,
        lines: vec![check(false, "ran join", "exit 2\nCONFLICT a.txt"), Line::Log("one\ntwo".to_string())],
    }];
    let expected = "mortar acceptance report\n\
                    commit 0123456789abcdef0123456789abcdef01234567 with local changes\n\
                    \n\
                    Step 1 the flow: FAIL\n  \
                    FAIL  ran join: exit 2\n        \
                    CONFLICT a.txt\n  \
                    log   one\n        \
                    two\n\
                    \n\
                    Result: FAIL\n";
    assert_eq!(format_report("commit 0123456789abcdef0123456789abcdef01234567 with local changes", &blocks), expected);
}

#[test]
fn no_blocks_is_the_title_the_commit_and_a_passing_result() {
    assert_eq!(
        format_report("commit not read: git rev-parse HEAD exited 128: no HEAD", &[]),
        "mortar acceptance report\ncommit not read: git rev-parse HEAD exited 128: no HEAD\n\nResult: PASS\n"
    );
}
