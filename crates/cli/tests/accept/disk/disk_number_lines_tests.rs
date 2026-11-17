use super::{disk_number_lines, DiskNumbers};
use crate::report::{block_verdict, Checks, Line, Verdict};
use crate::stage::failed_line;
use std::time::Duration;

#[test]
fn taken_numbers_make_five_ordered_passing_lines() {
    let numbers = DiskNumbers {
        checkout: Ok(1_048_576), zone_folder: Ok(0), request_folder: Ok(1_572_864),
        store: Ok(123_456_789), snapshot: Ok(Duration::from_micros(1_234_567)),
    };
    let mut checks = Checks::new();
    disk_number_lines(&numbers, &mut checks);
    let lines = checks.lines();
    assert_eq!(lines, vec![
        Line::Check { ok: true, did: "number: a full checkout of the copy (HEAD's files, apparent bytes)".to_string(), saw: "1048576 bytes (1.0 MiB)".to_string() },
        Line::Check { ok: true, did: "number: the mr folder zone folder (its files, apparent bytes, .git and target left out)".to_string(), saw: "0 bytes (0.0 MiB)".to_string() },
        Line::Check { ok: true, did: "number: the mr folder request folder (its files, apparent bytes, .git and target left out)".to_string(), saw: "1572864 bytes (1.5 MiB)".to_string() },
        Line::Check { ok: true, did: "number: mr disk's object store, counted once".to_string(), saw: "123456789 bytes (117.7 MiB)".to_string() },
        Line::Check { ok: true, did: "number: mr snapshot's wall time in the request folder after a small edit".to_string(), saw: "1234.6 ms".to_string() },
    ]);
    assert_eq!(block_verdict(&lines, None), Verdict::Pass);
}

#[test]
fn untaken_numbers_make_five_ordered_failed_lines_with_their_reasons() {
    let numbers = DiskNumbers {
        checkout: Err("tree".to_string()), zone_folder: Err("zone".to_string()),
        request_folder: Err("request".to_string()), store: Err("disk".to_string()),
        snapshot: Err("snapshot".to_string()),
    };
    let mut checks = Checks::new();
    disk_number_lines(&numbers, &mut checks);
    let lines = checks.lines();
    assert_eq!(lines, vec![
        failed_line("number: a full checkout of the copy (HEAD's files, apparent bytes)", "not taken: tree"),
        failed_line("number: the mr folder zone folder (its files, apparent bytes, .git and target left out)", "not taken: zone"),
        failed_line("number: the mr folder request folder (its files, apparent bytes, .git and target left out)", "not taken: request"),
        failed_line("number: mr disk's object store, counted once", "not taken: disk"),
        failed_line("number: mr snapshot's wall time in the request folder after a small edit", "not taken: snapshot"),
    ]);
    assert_eq!(block_verdict(&lines, None), Verdict::Fail);
}

#[test]
fn any_single_untaken_number_fails_the_step() {
    for missing in 0..5 {
        let mut numbers = DiskNumbers {
            checkout: Ok(1), zone_folder: Ok(2), request_folder: Ok(3), store: Ok(4),
            snapshot: Ok(Duration::from_millis(5)),
        };
        match missing {
            0 => numbers.checkout = Err("missing".to_string()),
            1 => numbers.zone_folder = Err("missing".to_string()),
            2 => numbers.request_folder = Err("missing".to_string()),
            3 => numbers.store = Err("missing".to_string()),
            _ => numbers.snapshot = Err("missing".to_string()),
        }
        let mut checks = Checks::new();
        disk_number_lines(&numbers, &mut checks);
        assert_eq!(block_verdict(&checks.lines(), None), Verdict::Fail, "missing number {missing}");
    }
}
