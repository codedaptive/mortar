//! 03-kill.md's "Consistent" list, checked after every case. A const only: case_lines_tests.rs reads it through
//! CaseLines.

pub const CONSISTENT_ITEMS: [&str; 5] = [
    "consistent: git fsck clean",
    "consistent: no lock left held",
    "consistent: every folder's HEAD and index match its branch",
    "consistent: no open intent left in the op log",
    "consistent: repeating the finished command once more moves nothing",
];
