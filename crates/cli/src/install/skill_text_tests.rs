//! Whole installed skill text for both command names.

use super::skill_text;

#[test]
fn mr_skill_matches_specification() {
    assert_eq!(skill_text("mr"), r#"---
name: mortar
description: Mortar (mr) keeps version-control history for agent teams with plain git. Use it when you work in a Mortar request folder (mr status names your request and zone), or when told to save, join or resolve your work with mr.
---

# Mortar: the agent verbs

You work in a request folder: a git worktree Mortar made for one request in one zone. Mortar saves, joins and
resolves your work. You never type a git write command (commit, merge, rebase, reset, checkout, switch, push, pull,
stash, branch, tag, update-ref). Reading with git (status, diff, log, show) is fine.

Every mr command prints where things stand, and its last line is `next:`, the one step to take. When unsure, run
`mr status` and do what its `next:` says.

| Verb | What it does |
|---|---|
| `mr status` | Where you are, what you changed, whether your zone moved, anything conflicted, and the next step |
| `mr save "<sentence>"` | Keeps your work as a commit. The sentence is one line saying what the work does. No staging, no branch names |
| `mr join` | Joins your request to the zone. If the zone moved, your work is rebased onto it: run your unit tests again, then `mr join` again |
| `mr resolve` | Writes the conflicted files into your folder with markers. Fix them, then run it again |
| `mr need <path>` | Adds a path to your folder when the work needs a file outside it. A path outside your work order is recorded as a gap |
| `mr undo` | Takes back your own last step, in your own folder only |

## A request, start to finish

1. `mr status`.
2. Make your change, then `mr save "<what it does>"`.
3. `mr join`.
   - Joined: your part is done. The zone's tests run next.
   - Rebased: run your unit tests, then `mr join` again.
   - Conflicted: `mr join` lists the files and leaves your folder as it was. Run `mr resolve`: it writes each
     conflicted file into your folder with markers (`<<<<<<<`, `=======`, `>>>>>>>`). Fix each file and remove
     every marker, then run `mr resolve` again. It rebases your work and says `next: mr join`.

## Rules

- The six verbs above are yours. The others (`mr folder`, `mr squash`, `mr land`, `mr publish` and the rest) belong
  to the harness and the landing script.
- Unsaved work is never a problem: `mr join` saves it first.
- Every mr command can be repeated safely. A refusal says why, whose the thing is, and what to do.
- `mr help` lists every verb; `mr <verb> --help` shows one verb's usage.
"#);
}

#[test]
fn mortar_skill_matches_specification() {
    assert_eq!(skill_text("mortar"), r#"---
name: mortar
description: Mortar (mortar) keeps version-control history for agent teams with plain git. Use it when you work in a Mortar request folder (mortar status names your request and zone), or when told to save, join or resolve your work with mortar.
---

# Mortar: the agent verbs

You work in a request folder: a git worktree Mortar made for one request in one zone. Mortar saves, joins and
resolves your work. You never type a git write command (commit, merge, rebase, reset, checkout, switch, push, pull,
stash, branch, tag, update-ref). Reading with git (status, diff, log, show) is fine.

Every mortar command prints where things stand, and its last line is `next:`, the one step to take. When unsure, run
`mortar status` and do what its `next:` says.

| Verb | What it does |
|---|---|
| `mortar status` | Where you are, what you changed, whether your zone moved, anything conflicted, and the next step |
| `mortar save "<sentence>"` | Keeps your work as a commit. The sentence is one line saying what the work does. No staging, no branch names |
| `mortar join` | Joins your request to the zone. If the zone moved, your work is rebased onto it: run your unit tests again, then `mortar join` again |
| `mortar resolve` | Writes the conflicted files into your folder with markers. Fix them, then run it again |
| `mortar need <path>` | Adds a path to your folder when the work needs a file outside it. A path outside your work order is recorded as a gap |
| `mortar undo` | Takes back your own last step, in your own folder only |

## A request, start to finish

1. `mortar status`.
2. Make your change, then `mortar save "<what it does>"`.
3. `mortar join`.
   - Joined: your part is done. The zone's tests run next.
   - Rebased: run your unit tests, then `mortar join` again.
   - Conflicted: `mortar join` lists the files and leaves your folder as it was. Run `mortar resolve`: it writes each
     conflicted file into your folder with markers (`<<<<<<<`, `=======`, `>>>>>>>`). Fix each file and remove
     every marker, then run `mortar resolve` again. It rebases your work and says `next: mortar join`.

## Rules

- The six verbs above are yours. The others (`mortar folder`, `mortar squash`, `mortar land`, `mortar publish` and the rest) belong
  to the harness and the landing script.
- Unsaved work is never a problem: `mortar join` saves it first.
- Every mortar command can be repeated safely. A refusal says why, whose the thing is, and what to do.
- `mortar help` lists every verb; `mortar <verb> --help` shows one verb's usage.
"#);
}
