//! WO-orc-snapshot-it-6 / #711: metadata absence/directory outcomes on NEW candidates, not paths exempted by HEAD.
use crate::support::{blob, write, Fixture};
use snapshot::snapshot;
use std::fs;
use testkit::write_git_shim;

#[test]
fn new_candidates_deleted_under_a_replaced_parent_or_now_plain_directories_do_not_become_size_refusals() {
    // Joins real snapshot_tree -> real changed-path output -> filesystem event -> file_size -> capped snapshot.
    // Git cannot list an empty plain directory by itself. After real diff lists a new file we preserve it in .git,
    // then make its metadata missing, under a regular-file parent, or an ordinary directory before the size probe.
    for kind in ["deleted", "under-file", "directory"] {
        let f = Fixture::new(&[]);
        let path = if kind == "under-file" { "probe/leaf" } else { "probe" };
        let bytes = vec![b'p'; 64];
        write(&f.repo.path, path, &bytes);
        let index = fs::read(f.common.join("index")).unwrap();
        let source = f.repo.path.join("probe");
        let backup = f.common.join("preserved-probe");
        let marker = f.common.join("probe-event-done");
        let shim = f.common.join("probe-event.sh");
        write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = diff ] && mkdir \"$MORTAR_NONE_ONCE\" 2>/dev/null; then\n  git \"$@\"\n  mv \"$MORTAR_NONE_SOURCE\" \"$MORTAR_NONE_BACKUP\"\n  case \"$MORTAR_NONE_KIND\" in\n    under-file) printf 'replacement parent\\n' >\"$MORTAR_NONE_SOURCE\";;\n    directory) mkdir \"$MORTAR_NONE_SOURCE\";;\n    deleted) :;;\n    *) exit 97;;\n  esac\n  printf 'done\\n' >\"$MORTAR_NONE_MARKER\"\n  exit 0\nfi\nexec git \"$@\"\n").unwrap();
        let mut git = f.git.clone();
        git.program = shim;
        git.env.extend([
            ("MORTAR_NONE_ONCE".into(), f.common.join("probe-once").into_os_string()),
            ("MORTAR_NONE_SOURCE".into(), source.clone().into_os_string()),
            ("MORTAR_NONE_BACKUP".into(), backup.clone().into_os_string()),
            ("MORTAR_NONE_MARKER".into(), marker.clone().into_os_string()),
            ("MORTAR_NONE_KIND".into(), kind.into()),
        ]);
        let saved = snapshot(&git, &f.repo.path, &f.target, &[], 0).unwrap();
        assert!(saved.written);
        assert!(saved.refused.is_empty(), "{kind}: {:?}", saved.refused);
        assert_eq!(blob(&f.git, &saved.tree, path), bytes, "the real full tree captured the original candidate before the metadata event");
        assert_eq!(fs::read(marker).unwrap(), b"done\n");
        assert_eq!(fs::read(if kind == "under-file" { backup.join("leaf") } else { backup }).unwrap(), bytes);
        assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
        assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
        match kind {
            "deleted" => assert!(!source.exists()),
            "under-file" => assert_eq!(fs::read(source).unwrap(), b"replacement parent\n"),
            "directory" => { assert!(source.is_dir()); assert!(!source.join(".git").exists()); }
            _ => unreachable!(),
        }
    }
    // Regular-file cap enforcement has separate positive controls in caps.rs. These candidates are not in HEAD,
    // so HEAD membership cannot mask a bogus refusal.
}
