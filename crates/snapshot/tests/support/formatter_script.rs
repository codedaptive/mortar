use super::Fixture;
use snapshot::Formatter;
use std::path::PathBuf;
use testkit::write_git_shim;

/// Real child program: records literal argv/cwd/call count, then performs the test's actual formatting or refusal.
pub fn formatter_script(f: &Fixture, body: &[u8]) -> (Formatter, PathBuf) {
    let program = f.common.join("formatter script.sh");
    let trace = f.common.join("formatter trace");
    let mut script = b"#!/bin/sh\nset -eu\ntrace=$1\nshift\nprintf '%s\\0' \"$@\" >\"$trace\"\npwd -P >\"$trace.cwd\"\nprintf 'run\\n' >>\"$trace.count\"\ntest \"$1\" = 'literal $(touch unexpected)'\nshift\ntest \"$1\" = --\nshift\n".to_vec();
    script.extend_from_slice(body);
    write_git_shim(&program, &script).unwrap();
    (Formatter { program, args: vec![trace.clone().into_os_string(), "literal $(touch unexpected)".into(), "--".into()], timeout: std::time::Duration::from_secs(60) }, trace)
}
