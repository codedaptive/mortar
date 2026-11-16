use crate::{json::{check, Report}, observe::peer, support::Fixture};
use std::path::PathBuf;

pub struct Need { pub f: Fixture, pub folder: PathBuf }
impl Need {
    pub fn new() -> Self {
        let f = Fixture::new();
        let folder = peer(&f, "need", &["src".into()]);
        assert!(!folder.join("sub/keep").exists(), "need must really materialize a previously excluded path");
        Self { f, folder }
    }
    pub fn report(&self, args: &[&str], exit: i32, outcome: &str) -> Report {
        check(self.f.command(&self.folder, args).output().unwrap(), "need", exit, outcome)
    }
    pub fn paths(&self) -> Vec<String> {
        folders::make::read_folder_record(&self.f.git.git_dir(&self.folder).unwrap()).unwrap().unwrap().needed_paths
    }
}
