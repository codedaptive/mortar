//! Real Gitio objects and literal landing-state inputs for the public Land -> Oplog RITs.
use crate::{owed_ops::{locked, raw, refs}, support::Fixture};
use gitio::{Oid, RunOpts};
use land::{BatchLandAnswer, BatchMember, Squash};
use oplog::ChangeId;
use std::{fs, path::PathBuf};

pub(super) const IDS: [&str; 3] = [
    "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "llllllllllllllllllllllllllllllll", "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
];
pub(super) const ACTOR: &str = "batch-rit";
pub(super) const INTEGRATION: &str = "refs/heads/integration";
pub(super) const MAIN: &str = "refs/heads/main";
pub(super) struct Batch {
    pub f: Fixture, pub origin: Oid, pub squashes: Vec<Squash>, pub landed: Vec<Oid>, pub path: PathBuf,
    zones: Vec<String>, json_second: String,
}
impl Batch {
    pub fn new(second: &str, json_second: &str) -> Self {
        let f = Fixture::new(); let base = f.git.read_ref(MAIN).unwrap().unwrap();
        let base_tree = f.git.commit_info(&base).unwrap().tree;
        let tree = f.git.commit_info(&f.source).unwrap().tree;
        let origin = f.git.commit_tree(&base_tree, std::slice::from_ref(&base), "independent origin\n", None, None).unwrap();
        let zones = vec!["core".into(), second.into(), "third".into()];
        let mut squashes: Vec<Squash> = vec![]; let mut landed: Vec<Oid> = vec![];
        for (i, zone) in zones.iter().enumerate() {
            let parent = if i == 0 { &base } else { &squashes[i - 1].commit };
            let commit = f.git.commit_tree(&tree, std::slice::from_ref(parent),
                &format!("fixture squash\n\nMortar-Change-Id: {}\nMortar-Zone: {zone}\n", IDS[i]), None, None).unwrap();
            let onto = if i == 0 { &origin } else { &landed[i - 1] };
            // Every squash after the first has the same tree. The independent origin forces merge commits;
            // these objects are the expected targets, built without the Land merge or state writer.
            landed.push(f.git.commit_tree(&tree, &[onto.clone(), commit.clone()],
                &format!("land {} ({zone})\n", IDS[i]), None, None).unwrap());
            squashes.push(Squash { commit, change_id: ChangeId::parse(IDS[i]).unwrap(), source: f.source.clone(), requests: vec![] });
        }
        raw(&f, &["update-ref", MAIN, origin.as_str()]); raw(&f, &["update-ref", INTEGRATION, origin.as_str()]);
        let path = f.common.join("mortar/landing.json");
        Self { f, origin, squashes, landed, path, zones, json_second: json_second.into() }
    }
    pub fn text(&self, count: usize, pending: Option<&Oid>) -> String {
        let members = (0..count).map(|i| {
            let zone = if i == 1 { &self.json_second } else { &self.zones[i] };
            format!("{{\"change_id\":\"{}\",\"zone\":\"{zone}\",\"squash\":\"{}\",\"integration\":\"{}\"}}",
                IDS[i], self.squashes[i].commit, self.landed[i])
        }).collect::<Vec<_>>().join(",");
        let pending = pending.map_or(String::new(), |id| format!(",\"pending\":\"{id}\""));
        format!("{{\"origin_at_start\":\"{}\",\"landings\":[{members}]{pending}}}\n", self.origin)
    }
    pub fn legacy(&self, pending: Option<&Oid>) -> String {
        let pending = pending.map_or(String::new(), |id| format!(",\"pending\":\"{id}\""));
        format!("{{\"change_id\":\"kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk\",\"zone\":\"core\",\"squash\":\"{}\",\"origin_at_start\":\"{}\",\"integration\":\"{}\"{pending}}}\n",
            self.squashes[0].commit, self.origin, self.landed[0])
    }
    pub fn seed(&self, text: &str, tip: &Oid) {
        fs::create_dir_all(self.path.parent().unwrap()).unwrap(); fs::write(&self.path, text).unwrap();
        raw(&self.f, &["update-ref", INTEGRATION, tip.as_str()]);
    }
    pub fn land(&self, i: usize) {
        assert_eq!(locked(&self.f, || land::land_batch(&self.f.git, &refs(), &self.squashes[i], ACTOR)),
            BatchLandAnswer::Landed { integration: self.landed[i].clone() });
    }
    pub fn members(&self, count: usize) -> Vec<BatchMember> {
        (0..count).map(|i| BatchMember { change_id: ChangeId::parse(IDS[i]).unwrap(), zone: self.zones[i].clone() }).collect()
    }
    pub fn observe(&self) -> (Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>) {
        (self.f.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout,
            fs::read(oplog::oplog_path(&self.f.common)).ok(), fs::read(&self.path).ok())
    }
    pub fn pending(&self, member: usize) -> Oid {
        let tree = self.f.git.commit_info(&self.landed[member]).unwrap().tree;
        self.f.git.commit_tree(&tree, std::slice::from_ref(&self.landed[member]), "named files\n", None, None).unwrap()
    }
}
