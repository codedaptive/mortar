//! CLI run_guard_install/path_warning_for -> real Land guard_install and Gitio,
//! with actual Cargo binaries, PATH lookup and marker-file reads. Each Fixture owns/removes its whole repository
//! and scratch executables. The expected full plain reports and hook bytes are independent protocol literals.
use crate::support::{script, shim_path, Fixture};
use std::{fs, path::{Path, PathBuf}, process::{Command, Stdio}};

fn mortar() -> PathBuf { Path::new(env!("CARGO_BIN_EXE_mortar")).canonicalize().unwrap() }

fn bin(f: &Fixture, name: &str) -> PathBuf {
    let dir = f.root.join(name); fs::create_dir(&dir).unwrap(); dir
}

fn foreign_mr(dir: &Path) {
    // If production probes this candidate by running it, the sentinel makes the test fail even if it ignores exit 97.
    script(&dir.join("mr"), b"#!/bin/sh\nprintf 'foreign mr ran\\n' >\"$MORTAR_VALIDATION_FOREIGN_RAN\"\nexit 97\n");
}

fn copied_mortar(dir: &Path, name: &str) -> PathBuf {
    let copied = dir.join(name); fs::copy(mortar(), &copied).unwrap();
    assert!(!fs::symlink_metadata(&copied).unwrap().file_type().is_symlink());
    assert_ne!(copied.canonicalize().unwrap(), mortar()); copied
}

fn install(f: &Fixture, program: &Path, first: &Path, warns: bool) {
    let program = program.canonicalize().unwrap();
    let before = f.observe();
    // Reuse the shared fixture's isolated environment/arguments even when running a relocated Mortar binary.
    let template = f.command(&f.request, &["--actor", "guard-install-path-lookup-harness", "guard", "install"]);
    let output = Command::new(&program).current_dir(template.get_current_dir().unwrap())
        .args(template.get_args()).env_clear()
        .envs(template.get_envs().filter_map(|(key, value)| value.map(|value| (key, value))))
        .env("PATH", shim_path(first)).env("MORTAR_VALIDATION_FOREIGN_RAN", f.root.join("foreign-ran"))
        .stdin(Stdio::null()).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let warning = if warns {
        format!("the first mr on PATH is {}, not this Mortar: run Mortar as mortar ({})\n",
            first.join("mr").display(), program.display())
    } else { String::new() };
    assert_eq!(String::from_utf8(output.stdout).unwrap(), format!(
        "the guard is installed in {}\nit guards refs/heads/integration and refs/heads/main, and pushes of them to origin\n{}next: nothing to do: git now refuses moves of integration and main that Mortar did not make\n",
        f.common.join("hooks").display(), warning));
    assert_eq!(fs::read(f.common.join("mortar/guard.json")).unwrap(),
        b"{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}\n");
    assert_eq!(fs::read(f.common.join("hooks/reference-transaction")).unwrap(),
        b"#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n");
    assert_eq!(fs::read(f.common.join("hooks/pre-push")).unwrap(),
        b"#!/bin/sh\nexec mortar guard-hook pre-push \"$@\"\n");
    assert_eq!(f.observe(), before, "install must not move refs or alter the request/index/op log");
    assert!(!f.root.join("foreign-ran").exists(), "the PATH candidate must be read, never executed");
}

#[test]
fn foreign_mr_first_warns_last_before_next_but_installs_the_real_guard_without_running_the_candidate() {
    let f = Fixture::new(); let first = bin(&f, "foreign-bin"); foreign_mr(&first);
    install(&f, &mortar(), &first, true);
}

#[test]
fn this_builds_own_bin_first_installs_without_a_path_warning() {
    let f = Fixture::new();
    let built_mr = Path::new(env!("CARGO_BIN_EXE_mr")).canonicalize().unwrap();
    let first = built_mr.parent().unwrap();
    assert_eq!(first.join("mortar").canonicalize().unwrap(), mortar());
    install(&f, &mortar(), first, false);
}

#[test]
fn a_copied_mortar_at_another_path_is_recognized_by_its_actual_marker_without_warning() {
    let f = Fixture::new(); let first = bin(&f, "copied-bin");
    let copied = copied_mortar(&first, "mr"); assert_ne!(copied, mortar());
    install(&f, &mortar(), &first, false);
}

#[test]
fn a_myrepos_like_mr_beside_the_running_mortar_warns_last_before_next() {
    let f = Fixture::new(); let first = bin(&f, "neighbor-bin");
    let relocated = copied_mortar(&first, "mortar"); foreign_mr(&first);
    assert_eq!(relocated.parent(), Some(first.as_path()));
    install(&f, &relocated, &first, true);
}
