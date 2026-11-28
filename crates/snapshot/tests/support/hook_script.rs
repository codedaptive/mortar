use super::Fixture;
use std::fs;
use testkit::write_git_shim;

/// Installs a real commit hook: `#!/bin/sh` and `body`, executable, at the common dir's `hooks/<name>`, the folder
/// git looks in when core.hooksPath is unset (the fixture's config sets none).
pub fn hook_script(f: &Fixture, name: &str, body: &[u8]) {
    let hooks = f.common.join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    let mut script = b"#!/bin/sh\n".to_vec();
    script.extend_from_slice(body);
    write_git_shim(&hooks.join(name), &script).unwrap();
}
