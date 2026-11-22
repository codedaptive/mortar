use super::{build_number_lines, ZoneNumbers};
use crate::report::{block_verdict, Checks, Line, Verdict};
use std::time::Duration;

fn numbers(build: Result<Duration, String>, built_request: Result<u64, String>) -> ZoneNumbers {
    ZoneNumbers {
        zone_folder: Ok(1),
        build,
        request_folder: Ok(2),
        built_request,
        store: Ok(3),
        snapshot: Ok(Duration::from_millis(4)),
    }
}

fn log(text: &str) -> Line {
    Line::Log(text.to_string())
}

#[test]
fn build_number_lines_logs_the_build_and_the_request_folder_after_it() {
    let rows: Vec<(&str, ZoneNumbers, Vec<Line>)> = vec![
        (
            "both taken",
            numbers(Ok(Duration::from_micros(1_234_567)), Ok(1_572_864)),
            vec![
                log("number: Z: the debug build's wall time: 1234.6 ms"),
                log("number: Z: the request folder made after the build (its files and build output, apparent bytes, .git left out): 1572864 bytes (1.5 MiB)"),
            ],
        ),
        (
            "the request folder not measured",
            numbers(Ok(Duration::from_millis(2)), Err("no folder".to_string())),
            vec![
                log("number: Z: the debug build's wall time: 2.0 ms"),
                log("number: Z: the request folder made after the build (its files and build output, apparent bytes, .git left out): not taken: no folder"),
            ],
        ),
        (
            "the build not taken",
            numbers(Err("cargo build: exit 101".to_string()), Err("ignored".to_string())),
            vec![log("number: Z: the debug build: not taken: cargo build: exit 101")],
        ),
    ];
    for (name, numbers, expected) in rows {
        // One passing check first: a block with no check fails, and these lines must not change that verdict.
        let mut checks = Checks::new();
        checks.check("a check that held", true, "held");
        build_number_lines("Z", &numbers, &mut checks);
        let lines = checks.lines();
        assert_eq!(lines[1..], expected[..], "{name}");
        assert_eq!(block_verdict(&lines, None), Verdict::Pass, "{name}");
    }
}
