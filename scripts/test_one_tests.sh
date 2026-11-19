#!/bin/sh
# test_one_tests.sh: checks test_one.sh's path handling. Each case runs test_one.sh with CARGO=echo, so cargo is
# never called and no test runs, and compares its whole output and exit code with what they must be.
# Run from the repository root (make script-test).
set -u
here=$(dirname "$0")
fails=0

# check NAME DIR GROUP EXIT OUTPUT: test_one.sh exits EXIT and prints exactly OUTPUT. CARGO_TEST_ARGS is cleared, since
# test_one.sh reads it and a value left in the environment would change the command it prints.
check() {
    out=$(CARGO_TEST_ARGS= CARGO=echo sh "$here/test_one.sh" "$2" "$3" 2>&1); rc=$?
    if [ "$rc" = "$4" ] && [ "$out" = "$5" ]; then echo "ok    $1"
    else echo "FAIL  $1: exit $rc (want $4), output: $out (want: $5)"; fails=$((fails + 1)); fi
}

module_refusal="test-one: a module path with a segment ending in integration has no unit tests here; use DIR=crates/cli GROUP=integration"
target_refusal="test-one: DIR must name a test target, not a file inside one; use DIR=crates/cli/tests/cli_rit"

check "a module named integration refuses" crates/cli/src/install/integration unit 2 "$module_refusal"
check "a module ending in integration refuses" crates/cli/src/foo_integration unit 2 "$module_refusal"
check "a module under one ending in integration refuses" crates/cli/src/x_integration/y unit 2 "$module_refusal"
check "a module starting with integration runs" crates/folders/src/integration_folder unit 0 "test -p folders --lib integration_folder:: -- --skip integration::"
check "a plain module runs" crates/cli/src/install unit 0 "test -p cli --lib install:: -- --skip integration::"
check "a path inside a test target refuses" crates/cli/tests/cli_rit/rit9 integration 2 "$target_refusal"
check "a double slash inside a target names the target" crates/cli/tests//cli_rit/rit9 integration 2 "$target_refusal"
check "a double and trailing slash runs the target" crates/cli/tests//cli_rit/ integration 0 "test -p cli --test cli_rit"
check "a plain target runs" crates/cli/tests/cli_rit integration 0 "test -p cli --test cli_rit"

[ "$fails" = 0 ] && echo "test_one_tests: all passed" || { echo "test_one_tests: $fails failed"; exit 1; }
