use land::LandRefs;

pub fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}
