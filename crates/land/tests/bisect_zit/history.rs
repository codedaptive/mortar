//! The bisect ZITs' throwaway history: L0 passes, then five landings on main, the third of which can break check.sh.

use std::path::Path;
use std::process::Command;

use gitio::{Git, Oid};
use testkit::TestRepo;

/// The change IDs of the landings' squashes: L1, L2, the third's squash, L4, L5.
pub const IDS: [&str; 5] = [
    "kzlymxnwovpuqtrssrtqupvowxnymlzk",
    "zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk",
    "mmmmnnnnoooopppqqqqrrrrssssttttu",
    "kkkkllllmmmmnnnnoooopppprrrrssss",
    "ttttuuuuvvvvwwwwxxxxyyyyzzzzkkkk",
];

/// How the third landing lands.
pub enum Third {
    /// A squash S on a side branch from L2, with its own trailers (zone "zone-three") and this check.sh, merged
    /// onto main with --no-ff and a message without trailers: parents [L2, S].
    Merge(&'static str),
    /// A fast-forward commit with this check.sh and no trailers.
    PlainFastForward(&'static str),
}

/// A built history: `landings` is [L0, L1, L2, L3, L4, L5]; `squash` is S for a merge landing.
pub struct History {
    pub repo: TestRepo,
    pub landings: Vec<Oid>,
    pub squash: Option<Oid>,
}

/// Builds L0 (check.sh "exit 0") and five landings; `fourth_script` is L4's check.sh when it changes it.
pub fn build(third: Third, fourth_script: Option<&str>) -> History {
    let repo = TestRepo::new().unwrap();
    let path = repo.path.clone();
    let mut landings = vec![commit(&path, &[("check.sh", "exit 0\n"), ("zero.txt", "0\n")], "L0", None)];
    landings.push(commit(&path, &[("one.txt", "1\n")], "L1", Some((IDS[0], "zone-one"))));
    landings.push(commit(&path, &[("two.txt", "2\n")], "L2", Some((IDS[1], "zone-two"))));
    let squash = match third {
        Third::Merge(script) => {
            run(&path, &["checkout", "-q", "-b", "side"]);
            let squash = commit(&path, &[("check.sh", script)], "S", Some((IDS[2], "zone-three")));
            run(&path, &["checkout", "-q", "main"]);
            run(&path, &["merge", "-q", "--no-ff", "-m", "Merge landing three", "side"]);
            landings.push(head(&path));
            Some(squash)
        }
        Third::PlainFastForward(script) => {
            landings.push(commit(&path, &[("check.sh", script)], "L3", None));
            None
        }
    };
    let mut fourth = vec![("four.txt", "4\n")];
    fourth.extend(fourth_script.map(|script| ("check.sh", script)));
    landings.push(commit(&path, &fourth, "L4", Some((IDS[3], "zone-four"))));
    landings.push(commit(&path, &[("five.txt", "5\n")], "L5", Some((IDS[4], "zone-five"))));
    History { repo, landings, squash }
}

/// A commit on L1 that is not on main.
pub fn off_main(history: &History) -> Oid {
    let path = &history.repo.path;
    let parent = history.landings[1].to_string();
    let tree = run(path, &["rev-parse", &format!("{parent}^{{tree}}")]);
    oid(&run(path, &["commit-tree", &tree, "-p", &parent, "-m", "off main"]))
}

/// The gitio value the bisect runs with. Its runs ignore the machine's global and system git config, so settings
/// such as core.hooksPath cannot reach the throwaway repository.
pub fn git_of(path: &Path) -> Git {
    let env = vec![("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into())];
    Git { program: "git".into(), repo: path.to_path_buf(), env }
}

/// Writes `files`, then commits them with `subject` and, when given, the change ID and zone trailers.
fn commit(path: &Path, files: &[(&str, &str)], subject: &str, trailers: Option<(&str, &str)>) -> Oid {
    for (name, text) in files {
        std::fs::write(path.join(name), text).unwrap();
        run(path, &["add", name]);
    }
    match trailers {
        Some((id, zone)) => {
            let block = format!("Mortar-Change-Id: {id}\nMortar-Zone: {zone}");
            run(path, &["commit", "-q", "-m", subject, "-m", &block])
        }
        None => run(path, &["commit", "-q", "-m", subject]),
    };
    head(path)
}

fn head(path: &Path) -> Oid {
    oid(&run(path, &["rev-parse", "HEAD"]))
}

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

/// Runs git in `path` with no global or system config, returning its stdout trimmed; panics when git fails.
pub fn run(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["-c", "user.name=Bisect Test", "-c", "user.email=bisect@example.invalid"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}
