//! WO-orc-gitio-it-4 / SPEC-gitio WO-gitio-1: inherited pathspec modes are cleared, explicit modes win.
use crate::support::GitRepo;
use gitio::{Git, RunOpts};
use std::{fs, path::PathBuf};
use testkit::test_child;

const MODES: [&str; 4] = [
    "GIT_NOGLOB_PATHSPECS", "GIT_LITERAL_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_ICASE_PATHSPECS",
];

#[test]
fn inherited_pathspec_modes_are_isolated_and_explicit_git_and_run_values_still_control_real_git() {
    // Joins Git::run's environment -> real path matching, and Git::snapshot_tree -> real excluded tree/index.
    // Only the subprocess inherits poison; no process-global set_var races with other tests.
    for inherited in [vec![MODES[0]], vec![MODES[1]], vec![MODES[2]], vec![MODES[3]], MODES.to_vec()] {
        let r = GitRepo::new();
        for (name, bytes) in [
            ("keep.txt", "base\n"), ("Mixed.TXT", "case\n"), ("glob[1].txt", "brackets\n"),
            ("glob1.txt", "digit\n"), ("dir/nested.txt", "nested\n"), ("excluded/tracked.txt", "old excluded\n"),
        ] {
            let path = r.path.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        r.git.run(&["add", "--all"], &RunOpts::default()).unwrap();
        r.git.run(&["commit", "-m", "pathspec fixture"], &RunOpts {
            env: vec![
                ("GIT_AUTHOR_NAME".into(), "Pathspec Fixture".into()),
                ("GIT_AUTHOR_EMAIL".into(), "pathspec@example.invalid".into()),
                ("GIT_AUTHOR_DATE".into(), "1700000000 +0000".into()),
                ("GIT_COMMITTER_NAME".into(), "Pathspec Fixture".into()),
                ("GIT_COMMITTER_EMAIL".into(), "pathspec@example.invalid".into()),
                ("GIT_COMMITTER_DATE".into(), "1700000000 +0000".into()),
            ],
            ..RunOpts::default()
        }).unwrap();
        fs::write(r.path.join("keep.txt"), b"new included\n").unwrap();
        fs::write(r.path.join("excluded/tracked.txt"), b"new excluded\n").unwrap();
        fs::write(r.path.join("excluded/new.txt"), b"excluded new file\n").unwrap();
        let mut child = test_child("pathspec_environment::pathspec_environment_child").unwrap();
        for key in MODES { child.env_remove(key); }
        for key in &inherited { child.env(key, "1"); }
        child.env("MORTAR_PATHSPEC_INHERITED", inherited.join(","));
        child.env("MORTAR_PATHSPEC_REPO", &r.path);
        child.env("MORTAR_PATHSPEC_CONTROL", "still-inherited");
        let output = child.output().unwrap();
        assert!(output.status.success(), "inherited {inherited:?}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
}

#[test]
#[ignore = "subprocess entry point; the parent supplies each real inherited pathspec mode and their incompatible combination"]
fn pathspec_environment_child() {
    let inherited = std::env::var("MORTAR_PATHSPEC_INHERITED").unwrap();
    assert_inherited(&inherited);
    let path = PathBuf::from(std::env::var_os("MORTAR_PATHSPEC_REPO").unwrap());
    let mut git = Git::new(&path);
    git.env.extend([("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into())]);
    let index = fs::read(path.join(".git/index")).unwrap();
    let head = git.rev_parse("HEAD").unwrap().unwrap();
    let saved = git.snapshot_tree(&head, &["excluded".into()]).unwrap();
    assert_eq!(git.run(&["show", &format!("{saved}:keep.txt")], &RunOpts::default()).unwrap().stdout, b"new included\n");
    assert_eq!(git.run(&["show", &format!("{saved}:excluded/tracked.txt")], &RunOpts::default()).unwrap().stdout, b"old excluded\n");
    assert_eq!(git.run(&["ls-tree", "-r", "--name-only", "-z", saved.as_str()], &RunOpts::default()).unwrap().stdout,
        b"Mixed.TXT\0dir/nested.txt\0excluded/tracked.txt\0glob1.txt\0glob[1].txt\0keep.txt\0");
    assert_eq!(fs::read(path.join(".git/index")).unwrap(), index);
    assert_eq!(git.rev_parse("HEAD").unwrap(), Some(head));
    assert_eq!(fs::read(path.join("excluded/tracked.txt")).unwrap(), b"new excluded\n");
    assert_eq!(fs::read(path.join("excluded/new.txt")).unwrap(), b"excluded new file\n");

    for (number, key) in MODES.iter().enumerate() {
        let (pattern, ordinary, enabled) = match *key {
            "GIT_NOGLOB_PATHSPECS" | "GIT_LITERAL_PATHSPECS" =>
                ("glob*.txt", b"glob1.txt\0glob[1].txt\0".as_slice(), b"".as_slice()),
            "GIT_GLOB_PATHSPECS" => ("*.txt",
                b"dir/nested.txt\0excluded/tracked.txt\0glob1.txt\0glob[1].txt\0keep.txt\0".as_slice(),
                b"glob1.txt\0glob[1].txt\0keep.txt\0".as_slice()),
            "GIT_ICASE_PATHSPECS" => ("mixed.txt", b"".as_slice(), b"Mixed.TXT\0".as_slice()),
            _ => unreachable!(),
        };
        for (stage, explicit, want) in [("cleared", None, ordinary), ("git", Some("1"), enabled), ("opts", Some("0"), ordinary)] {
            let trace = path.join(format!(".git/pathspec-{number}-{stage}.json"));
            let mut caller = git.clone();
            if stage != "cleared" { caller.env.push(((*key).into(), "1".into())); }
            let mut opts = RunOpts {
                read_only: true,
                env: vec![
                    ("GIT_TRACE2_EVENT".into(), trace.clone().into_os_string()),
                    ("GIT_TRACE2_ENV_VARS".into(), format!("{},MORTAR_PATHSPEC_CONTROL", MODES.join(",")).into()),
                ],
                ..RunOpts::default()
            };
            if stage == "opts" { opts.env.push(((*key).into(), "0".into())); }
            let listed = caller.run(&["ls-files", "-z", "--", pattern], &opts).unwrap();
            assert_eq!(listed.stdout, want, "{key} {stage} with inherited {inherited}");
            let events = fs::read_to_string(trace).unwrap();
            assert!(has_parameter(&events, "MORTAR_PATHSPEC_CONTROL", "still-inherited"), "{events}");
            for tracked in MODES {
                match explicit {
                    Some(value) if tracked == *key => assert!(has_parameter(&events, tracked, value), "{events}"),
                    _ => assert!(!events.contains(&format!("\"param\":\"{tracked}\"")), "inherited mode leaked: {events}"),
                }
            }
        }
    }
    assert_inherited(&inherited);
}

fn has_parameter(events: &str, key: &str, value: &str) -> bool {
    events.lines().any(|line| line.contains(&format!("\"param\":\"{key}\"")) && line.contains(&format!("\"value\":\"{value}\"")))
}

fn assert_inherited(inherited: &str) {
    for key in MODES {
        let expected = inherited.split(',').any(|name| name == key).then_some("1");
        assert_eq!(std::env::var(key).ok().as_deref(), expected, "caller process changed {key}");
    }
}
