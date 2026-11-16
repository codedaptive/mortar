use super::Ran;

fn ran(code: Option<i32>, signal: Option<i32>, stderr: &str) -> Ran {
    Ran { code, signal, stdout: String::new(), stderr: stderr.to_string() }
}

#[test]
#[ignore = "acceptance: make accept"]
fn exit_0_shows_no_stderr() {
    assert_eq!(ran(Some(0), None, "warning: noise\n").seen(), "exit 0");
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_failing_exit_shows_the_first_stderr_line_that_is_not_blank() {
    assert_eq!(ran(Some(2), None, "\n   \n  error: conflict in a.txt \nmore\n").seen(), "exit 2 / error: conflict in a.txt");
    assert_eq!(ran(Some(1), None, "").seen(), "exit 1");
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_signal_shows_the_signal() {
    assert_eq!(ran(None, Some(9), "").seen(), "killed by signal 9");
}
