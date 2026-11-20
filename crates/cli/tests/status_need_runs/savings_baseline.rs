use crate::{json::{self, Value}, support::Fixture};

#[test]
fn savings_large_status_diff_counts_both_git_reads() {
    let f = Fixture::new();
    f.write("src/file", (0..400).map(|i| format!("line {i} é substantial change\n")).collect::<String>().as_bytes());
    let b = super::savings::baseline(&f);
    let output = f.app(&["status"]);
    assert_eq!(output.status.code(), Some(0));
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let (line, body) = text.split_once("\n\n").unwrap();
    assert!(line.starts_with("[mr] tokens saved ≈ "), "{text}");
    assert!(line.contains("git status and git diff HEAD over this folder"), "{line}");
    assert!(line.strip_prefix("[mr] tokens saved ≈ ").unwrap().split_whitespace().next().unwrap().replace(',', "").parse::<usize>().unwrap() > 0);
    assert!(b > body.chars().count());
    let output = f.app(&["--json", "status"]);
    let raw = std::str::from_utf8(&output.stdout).unwrap();
    assert!(raw.contains(&format!("\"baselineChars\":{b},")), "{raw}");
    let report = json::read(output, "status", 0);
    assert!(report.object("savings").contains(&("baselineChars".into(), Value::Number(b as u64))));
}
