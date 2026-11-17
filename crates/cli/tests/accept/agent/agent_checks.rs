//! The titles of step 4's check lines, in order. A const only: agent_not_run_tests.rs reads it through
//! agent_not_run.

/// Index 0 is the conflict the drive sets up; index n (1 to 4) is 04-agent.md's check n.
pub const AGENT_CHECKS: [&str; 5] = [
    "the conflict: the drive's mr join in rc came before the agent's mr join, and the agent ran mr resolve",
    "1. zone a holds the agent's two edits, and src/a/a.txt keeps both sides' lines with no markers",
    "2. mr log --json shows the agent's actor a/r1 on save, join and resolve ops",
    "3. every git call in git.log whose parent is not mr or mortar is a read",
    "4. every mr or mortar call of the agent's in mr.log names one of the six verbs",
];
