//! savings_log_line: the exact snake_case line a savings.jsonl reader expects, with JSON-escaped strings.

use std::path::Path;

use super::savings_log_line;

#[test]
fn the_exact_line() {
    assert_eq!(
        savings_log_line("2025-11-20T02:40:00Z", Path::new("/w/req"), "status", 13_592, 540_252),
        "{\"time\":\"2025-11-20T02:40:00Z\",\"folder\":\"/w/req\",\"verb\":\"status\",\"output_tokens\":13592,\
         \"baseline_tokens\":540252}\n"
    );
}

#[test]
fn a_folder_with_a_quote_and_a_backslash_is_escaped() {
    assert_eq!(
        savings_log_line("1970-01-01T00:00:00Z", Path::new("/w/a\"b\\c"), "status", 0, 0),
        "{\"time\":\"1970-01-01T00:00:00Z\",\"folder\":\"/w/a\\\"b\\\\c\",\"verb\":\"status\",\"output_tokens\":0,\
         \"baseline_tokens\":0}\n"
    );
}
