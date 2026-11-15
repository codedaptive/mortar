use crate::{json::{check, Report}, support::*};
use std::{fs, path::{Path, PathBuf}};

pub fn record_path(f: &Fixture, folder: &Path) -> PathBuf {
    folders::record::record_path(&f.git.git_dir(folder).unwrap())
}

pub fn before(f: &Fixture, folder: &Path) -> Vec<Option<Vec<u8>>> {
    let dir = f.git.git_dir(folder).unwrap();
    vec![
        Some(raw(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"])),
        fs::read(dir.join("index")).ok(), fs::read(dir.join("index.lock")).ok(),
        fs::read(record_path(f, folder)).ok(), fs::read(oplog::oplog_path(&f.common)).ok(),
        fs::read(dir.join("mortar-join.json")).ok(), fs::read(folder.join("src/file")).ok(),
        fs::read(cli::config_path(&f.common)).ok(),
    ]
}

pub fn status(f: &Fixture, outcome: &str) -> Report {
    status_at(f, &f.request, outcome)
}

pub fn status_at(f: &Fixture, folder: &Path, outcome: &str) -> Report {
    let old = before(f, folder);
    let report = check(f.command(folder, &["--json", "status"]).output().unwrap(), "status", 0, outcome);
    assert_eq!(before(f, folder), old, "status must not change raw index/lock, refs, record, op log, pause or files");
    report
}

pub fn peer(f: &Fixture, name: &str, paths: &[String]) -> PathBuf {
    let folder = f.root.join(name);
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_request_folder(&setup(&f.git, &folder), "core", name, paths).unwrap();
    folder
}
