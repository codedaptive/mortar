//! Version::FLOOR: the oldest git mortar runs on.

use crate::Version;

impl Version {
    /// 2.46.0: the reference-transaction hook is given symbolic-ref writes (as `ref:<target>`), which land's guard
    /// reads to refuse them on the guarded refs; the builds checked from 2.42.0 to 2.45.4 pass none to the hook. mortar
    /// also runs `merge-tree --write-tree` with `--attr-source`, which works from 2.43.7 (2.42.0 crashes).
    pub const FLOOR: Version = Version {
        major: 2,
        minor: 46,
        patch: 0,
    };
}
