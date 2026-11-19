//! HARNESSES: the harnesses install can wire Mortar's skill into, in the order install lists them (spec 07 items 4
//! and 5).

use super::HarnessSpec;

/// claude, codex and cursor. Each is detected by its folder in HOME; its skills folder is where the harness reads
/// skills from (codex's is `.agents/skills`, not under `.codex`).
pub const HARNESSES: &[HarnessSpec] = &[
    HarnessSpec { name: "claude", folder: ".claude", skills: ".claude/skills" },
    HarnessSpec { name: "codex", folder: ".codex", skills: ".agents/skills" },
    HarnessSpec { name: "cursor", folder: ".cursor", skills: ".cursor/skills" },
];
