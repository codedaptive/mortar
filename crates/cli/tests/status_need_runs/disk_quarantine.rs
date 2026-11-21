use crate::{json::check, support::Fixture};
use std::fs;

#[test]
fn disk_leaves_no_object_quarantine_in_the_temp_dir() {
    let f = Fixture::new();
    f.write("src/file", b"unsaved request work\n");
    let temp = f.root.join("disk-temp");
    fs::create_dir(&temp).unwrap();
    assert_eq!(fs::read_dir(&temp).unwrap().count(), 0);

    let output = f.command(&f.request, &["--json", "--actor", "harness", "disk"]).env("TMPDIR", &temp).output().unwrap();
    let report = check(output, "disk", 0, "measured");
    assert!(report.strings("lines").contains(&format!(
        "{}: unsaved 1 tracked, 0 untracked; not joined to zone core", f.request.display()
    )));
    assert_eq!(fs::read_dir(&temp).unwrap().count(), 0);
}
