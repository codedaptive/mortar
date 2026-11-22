//! FOLDER_PARTS: the next parts that clear a refused or failed zone folder, indexed by folder_part.
//! A bare const; after_publish_report_tests cover each part.

/// The folder parts of next, in their fixed order: paths in the way, unmerged entries, an index matching no zone
/// commit, a git failure. Each says how to clear the folder so that a repeat can finish it.
pub const FOLDER_PARTS: [&str; 4] = [
    "move each path listed as in the way out of its zone folder, or put it back to that folder's index copy, and \
     leave the index as it is",
    "finish or undo the merge in each zone folder whose index has unmerged entries, leaving no unmerged entry",
    "take the staged changes out of each zone folder whose index matches no zone commit, without resetting its index \
     to HEAD alone",
    "clear what stopped git in each zone folder that failed",
];
