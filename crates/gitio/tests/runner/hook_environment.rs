use crate::support::TestRepo;
use gitio::RunOpts;
use std::fs;
use std::process::Command;

const HOOK_VARS: [&str; 6] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

#[test]
fn git_run_scrubs_inherited_hook_environment_without_mutating_parent() {
    // Joins Git::new -> Git::run -> git repository discovery and trace2 startup.
    // Put the hook environment in a separate process: no set_var race with parallel tests.
    let hook = TestRepo::new();
    let poison = hook.path.join("HOOK_ENV_MUST_NOT_LEAK");
    let mut child = Command::new(std::env::current_exe().unwrap());
    child.args([
        "--exact",
        "hook_environment::hook_environment_child",
        "--ignored",
        "--nocapture",
    ]);
    for key in HOOK_VARS {
        child.env(key, &poison);
    }
    child.env("MORTAR_GITIO_HOOK_CHILD", "1");
    child.env("MORTAR_GITIO_TRACE_CONTROL", "control-is-inherited");
    child.env("LC_ALL", "hook-locale");
    child.env("GIT_TERMINAL_PROMPT", "1");
    child.env("GIT_OPTIONAL_LOCKS", "1");
    let output = child.output().unwrap();
    assert!(output.status.success(), "child stdout:\n{}\nchild stderr:\n{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
#[ignore = "subprocess entry point; the parent test runs this with hook-only inherited environment"]
fn hook_environment_child() {
    assert_eq!(std::env::var("MORTAR_GITIO_HOOK_CHILD").unwrap(), "1");
    for key in HOOK_VARS {
        assert!(std::env::var_os(key).unwrap().to_string_lossy().contains("HOOK_ENV_MUST_NOT_LEAK"));
    }
    let repo = TestRepo::new();
    let trace = repo.path.join("trace.json");
    let tracked = format!("{},LC_ALL,GIT_TERMINAL_PROMPT,GIT_OPTIONAL_LOCKS,MORTAR_GITIO_TRACE_CONTROL", HOOK_VARS.join(","));
    let output = repo.git.run(&["rev-parse", "--show-toplevel"], &RunOpts {
        read_only: true,
        env: vec![
            ("GIT_TRACE2_EVENT".into(), trace.as_os_str().to_owned()),
            ("GIT_TRACE2_ENV_VARS".into(), tracked.into()),
        ],
        ..RunOpts::default()
    }).unwrap();
    let toplevel = String::from_utf8(output.stdout).unwrap();
    assert_eq!(std::path::Path::new(toplevel.trim_end()), repo.path.canonicalize().unwrap());
    let events = fs::read_to_string(trace).unwrap();
    assert!(!events.contains("HOOK_ENV_MUST_NOT_LEAK"), "{events}");
    for (key, value) in [
        ("MORTAR_GITIO_TRACE_CONTROL", "control-is-inherited"),
        ("LC_ALL", "C"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("GIT_OPTIONAL_LOCKS", "0"),
    ] {
        let parameter = format!("\"param\":\"{key}\"");
        let expected = format!("\"value\":\"{value}\"");
        assert!(events.lines().any(|line| line.contains(&parameter) && line.contains(&expected)),
            "trace2 did not report {key}={value}: {events}");
    }
    // The runner must scrub its child, not change the process that called it.
    for key in HOOK_VARS {
        assert!(std::env::var_os(key).unwrap().to_string_lossy().contains("HOOK_ENV_MUST_NOT_LEAK"));
    }
}
