use super::{kit_number_lines, ZoneNumbers};
use crate::report::{block_verdict, Checks, Line, Verdict};
use std::time::Duration;

type Kit = Result<(String, ZoneNumbers), String>;

fn log(text: &str) -> Line {
    Line::Log(text.to_string())
}

fn not_taken(why: &str) -> Result<u64, String> {
    Err(why.to_string())
}

#[test]
fn kit_number_lines_logs_each_kit_number_in_order() {
    let taken = ZoneNumbers {
        zone_folder: Ok(1_048_576),
        build: Ok(Duration::from_micros(1_234_567)),
        request_folder: Ok(0),
        built_request: Ok(1_572_864),
        store: Ok(123_456_789),
        snapshot: Ok(Duration::from_micros(2_500)),
    };
    let untaken = ZoneNumbers {
        zone_folder: not_taken("zone"),
        build: Err("build".to_string()),
        request_folder: not_taken("request"),
        built_request: not_taken("built"),
        store: not_taken("disk"),
        snapshot: Err("snapshot".to_string()),
    };
    let rows: Vec<(&str, Kit, Vec<Line>)> = vec![
        (
            "no kit",
            Err("HEAD holds no package folder".to_string()),
            vec![log("number: the one-kit zone: not taken: HEAD holds no package folder")],
        ),
        (
            "every number taken",
            Ok(("crates/small".to_string(), taken)),
            vec![
                log("number: the one-kit zone (crates/small): the mr folder zone folder (its files, apparent bytes, .git and target left out): 1048576 bytes (1.0 MiB)"),
                log("number: the one-kit zone (crates/small): the mr folder request folder (its files, apparent bytes, .git and target left out): 0 bytes (0.0 MiB)"),
                log("number: the one-kit zone (crates/small): the debug build's wall time: 1234.6 ms"),
                log("number: the one-kit zone (crates/small): the request folder made after the build (its files and build output, apparent bytes, .git left out): 1572864 bytes (1.5 MiB)"),
                log("number: the one-kit zone (crates/small): mr disk's object store, counted once: 123456789 bytes (117.7 MiB)"),
                log("number: the one-kit zone (crates/small): mr snapshot's wall time in the request folder after a small edit: 2.5 ms"),
            ],
        ),
        (
            "no number taken",
            Ok(("k".to_string(), untaken)),
            vec![
                log("number: the one-kit zone (k): the mr folder zone folder (its files, apparent bytes, .git and target left out): not taken: zone"),
                log("number: the one-kit zone (k): the mr folder request folder (its files, apparent bytes, .git and target left out): not taken: request"),
                log("number: the one-kit zone (k): the debug build: not taken: build"),
                log("number: the one-kit zone (k): mr disk's object store, counted once: not taken: disk"),
                log("number: the one-kit zone (k): mr snapshot's wall time in the request folder after a small edit: not taken: snapshot"),
            ],
        ),
    ];
    for (name, kit, expected) in rows {
        // One passing check first: a block with no check fails, and these lines must not change that verdict.
        let mut checks = Checks::new();
        checks.check("a check that held", true, "held");
        kit_number_lines(&kit, &mut checks);
        let lines = checks.lines();
        assert_eq!(lines[1..], expected[..], "{name}");
        assert_eq!(block_verdict(&lines, None), Verdict::Pass, "{name}");
    }
}
