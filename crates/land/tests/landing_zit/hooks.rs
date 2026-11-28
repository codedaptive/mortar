//! A landing merge does not run commit hooks.

use crate::fixture::{refs, Fixture, ACTOR, ID_A, ORIGIN_FILES, SQUASH_FILES};
use land::{land_batch, BatchLandAnswer};
use std::os::unix::fs::PermissionsExt;

#[test]
fn a_landing_merge_runs_no_hooks() {
    let fx = Fixture::new();
    let origin = fx.move_origin(&ORIGIN_FILES);
    let squash = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    // Install after building the squash, so these hooks test the landing only.
    for name in ["pre-commit", "commit-msg"] {
        let path = fx.repo.path.join(".git/hooks").join(name);
        std::fs::write(&path, "#!/bin/sh\nexit 7\n").unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let BatchLandAnswer::Landed { integration } = land_batch(&fx.git, &refs(), &squash, ACTOR).unwrap() else {
        panic!("a clean merge landing must answer Landed");
    };
    let info = fx.git.commit_info(&integration).unwrap();
    assert_eq!(info.parents, vec![origin, squash.commit]);
    assert_eq!(info.message, "land klmnopqrstuvwxyzklmnopqrstuvwxyz (land)\n");
    assert_eq!(fx.branch("integration"), integration);
}
