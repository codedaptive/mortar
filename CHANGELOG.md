# Changelog

All notable changes to Mortar are recorded here, in terms of what the program does for the people and agents who use it. Versions are the workspace's version in `Cargo.toml`.

## 0.1.0

The first public version. Mortar writes version-control history for teams of coding agents with plain git: it runs `git` as a program, keeps no store of its own, and every folder it makes is an ordinary git worktree.

### The agent verbs

- **Six verbs for an agent:** `mr status`, `mr save "<sentence>"`, `mr join`, `mr resolve`, `mr need <path>` and `mr undo`. They take no folder or name argument: the folder says who is asking.
- **No staging and no branch names.** `mr save` keeps the folder's work as one commit with the sentence as its subject.
- **Unsaved work is never a refusal.** `mr join` saves it first.
- **A join that meets a moved zone rebases** the request onto it and stops, so the tests can run again before the next `mr join`.
- **Conflicts are resolved in the folder.** A conflicted join lists the files and moves no branch beyond the save of any unsaved work. `mr resolve` writes them into the folder with markers, and finishes the rebase once they are fixed.
- **Undo for an agent's own steps:** its last save, or the last rebase or resolve step of a join. The folder as it stood is kept in a snapshot first. A join that has already moved the zone is not the agent's to undo.

### Folders

- **A folder per zone and per request,** each a sparse git worktree holding only its own paths and the files at the top of the repository.
- **`mr need <path>`** adds a path to a request's folder and records a path outside the request's own as a gap.
- **Attempt folders:** several tries at one request side by side, then keep one or drop them all.
- **A free-disk floor.** Mortar refuses to make a folder when free disk is under the floor (10 GB unless set).

### Saving

- **Change IDs.** A request keeps one change ID across its saves, and a zone's squash keeps one across re-squashes, carried in commit trailers.
- **A size cap for new files** (5 MiB unless set). A new file over the cap is left out and listed, not refused.
- **An optional formatter** runs over the changed files at each save, join and snapshot, under a timeout. A formatter that fails or runs too long is reported and the save goes ahead.
- **Snapshots:** `mr snapshot` keeps a folder's state on a hidden ref without a commit on its branch. Build output (`target`, `node_modules`, `.build`, and any folder named with `--build-dir`) is left out.

### Landing

- **`mr squash <zone>`** makes a zone's joined work one commit on the current origin.
- **`mr would-merge`** says whether a squash would merge into integration, without merging.
- **`mr rereview`** shows only what changed between a reviewed squash and a new one.
- **`mr land`, `mr back-out`, `mr publish`:** one landing at a time. Integration moves to the squash, or to a merge of it; a failed test puts integration back; a pass pushes it to the remote and moves the local origin. A remote that someone else moved meanwhile is fetched and the landing is put back onto it to land again.
- **`mr after-publish`** brings the other zones, and their folders, onto the new origin.
- **The origin branch only moves forward,** and a reviewed squash commit is never rewritten.

### The guard

- **`mr guard install`** adds two git hooks that refuse any move of the origin and integration branches, and any push of them to the configured remote, that Mortar did not make.
- It never overwrites a hook it did not write, and it fails closed when it cannot read its input or its config.

### The operation log, locks and recovery

- **Every move of a branch, a snapshot ref or a squash ref goes through an operation log** that records who asked and why, the old value and the new. `mr log` lists it; `mr restore` sets refs back as of an operation. The copies a fetch leaves under `refs/mortar/fetched/` and `refs/mortar-backup/` are not in it.
- **Every such move names the value it expects to replace,** so two movers cannot overwrite each other.
- **Locks:** one per zone for joins, one for landing, one for folder work. A lock whose holder on the same machine has died is taken over.
- **A killed run is settled by the next one.** Repeating the command finishes the work once.

### Upkeep

- **`mr overlap <zone>`** names the open requests that change the same paths, before anyone joins.
- **`mr disk`** reports what each folder, the object store and the refs cost; **`mr sweep`** deletes the refs finished work leaves behind and, with `--idle-days`, clears the build output of idle folders.
- **`mr backup` and `mr restore-backup`** mirror a machine's in-flight branches, snapshots and squashes to the remote and bring them back.
- **`mr bisect`** names the landing that broke a test, running the caller's command at each step in a throwaway worktree.

### Output

- **Every verb ends with `next:`,** the one step to take (except `--version`, and the guard hook when it allows a move). A refusal says why, who owns the thing in the way, and what to do.
- **`--json`** on any verb gives one JSON object with the outcome, the exit code and the next step (`--version --json` prints its own object, and `mortar --json guard-hook` prints nothing when it allows a move).
- **Fixed exit codes:** 0 done, 1 refused, 2 conflict, 3 busy, 4 usage, 5 failure.

### Install and help

- **`mortar install`** places the program in `~/.local/bin`, links `mr` to it unless another program owns that name, and offers a skill file for Claude Code, Codex and Cursor. **`mortar uninstall`** removes exactly what install placed.
- **`mortar --version`** prints the version and the commit.
- **`mr help`** lists every verb; `mr help <verb>` and `mr <verb> --help` show one verb's usage.

### Known limits

- Unix only. Mortar is built from source; there are no packaged releases.
- Licensed under the MIT License; see [`LICENSE`](LICENSE).
- The guard stops accidental moves of the protected branches. It is not a defence against someone who can write to the repository's git folder. See [`SECURITY.md`](SECURITY.md).
- `mr disk` and `mr sweep --idle-days` measure and clear the folder named `target` only, whatever `--build-dir` names.
- A relative `core.hooksPath` is read from the top of each worktree, so the guard does not cover a worktree whose path leads to a folder without the hooks.
