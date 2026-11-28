# Changelog

All notable changes to Mortar are recorded here, in terms of what the program does for the people and agents who use it. Versions are the workspace's version in `Cargo.toml`.

## Unreleased

### Exit codes

- **Exit codes follow CLI-SPEC v0.2:** 0 done, 1 conflict (the files are listed), 2 usage, 3 refused, 4 busy, 5 failure. They were 1 refused, 2 conflict, 3 busy, 4 usage. Every answer keeps its meaning; only the number moved. `mr help` and `mr --help` list the codes. `mortar guard-hook` refuses with exit 3, and git still stops the move. `mr join` on a zone that moved sideways is now refused (exit 3) with the zone's coordinator as owner, where it used to fail (exit 5) and ask for a repeat.
- **Configuration answers exit 2:** a command run outside a git repository (outcome `not-a-repository`), a missing Mortar config (`no-config`), a config or guard.json that cannot be read (`bad-config`), and `mortar guard-hook` with no guard.json (`no-guard-config`). They were refused (exit 3). `mr guard install` over a bad guard.json still rewrites it and exits 0.

### Landing

- **`mr would-merge <squash> --with <squash>...`** tries the squashes in order, each merged onto the one before, without merging anything. Exit 0 when every step merges; exit 1 names the step that conflicts and its files, and `--json` adds that step. Plain `mr would-merge` answers as before.
- **`mr commit-files <path>... [--message <text>] --actor <name>`** commits the named files, as the folder has them, as one commit. In a zone folder at its tip it moves `zone/<zone>` onto that commit; in the integration folder after `mr land` it moves integration, inside the landing, so `mr publish` pushes it and `mr back-out` drops it. Files that have not changed answer `same` and move nothing; if a stopped run left them staged, `same` names them with next `git restore --staged -- <paths>`. It is refused in a request or attempt folder (step: `mr save`), in a zone folder off its tip (step: `mr folder refresh`), and while a zone folder's index holds other staged changes. A repeat after a stop is safe. `--json`: outcome, commit, ref, moved, paths, and staged when the paths are still staged.

### Folders

- **`mr folder request ... --with <zone>...`** cuts a request folder on a merge of the zone's tip with each partner zone's tip, in the order given, so a request can build on work that has not landed yet. A merge that conflicts is refused, naming the partner and the files, and nothing is made; a partner already in the zone is named and not recorded. `--json` adds `partners` and `partners_in_zone`. Without `--with`, the folder is made as before.
- **`mr join` and `mr resolve` work from the request's recorded base**, so a request cut with partners replays only its own commits. When the zone has taken in the partners' work, the join answers as for any request; when a partner moved, the request is replayed onto the new merge and stops for its tests; a partner merge that now conflicts is refused, naming the partner and the files. Requests without partners join and resolve as before.
- **`mr status --json` in a request folder carries `base`** (the commit the request's own commits stand on), **`partners`** (`{zone, tip}` each, empty for a plain request) **and `partner_moved`**. The plain lines are unchanged.

## 0.3.0

### Landing

- **`mr squash <zone> --message <text>`** puts your text first in the squash's commit message, before the list of requests and Mortar's trailers, and Mortar still reads the squash back. A re-squash with a message keeps the squash's change ID; the same message again is already squashed. An empty or blank message is a usage error and writes nothing. Without `--message` the squash is made as before.
- **`mr withdraw <zone>`** takes back a zone's squash before it lands: it deletes the squash ref and names the commit and change ID it withdrew (`--json`: `withdrawn`). A repeat has nothing to withdraw and exits 0. A squash already in the origin branch is refused, as is the landing in flight, whose step is `mr back-out` first. The next squash draws a new change ID.
- **`mr publish --no-push`** moves only the local origin branch to integration and removes the squash ref: no push, and nothing is fetched. If the local origin moved meanwhile, the landing is put back onto it to land again. While an earlier publish's push is still open, `--no-push` is refused and a plain `mr publish` finishes it. `--json` on either form carries `pushed`, true only when this publish's push was confirmed.

### Folders

- **A new request folder gets its zone folder's build output as copy-on-write clones** (clonefile(2), never a byte copy): each configured build dir in the zone folder is cloned into the request folder, with the same files. The answer names each build dir not shared and why: the zone folder has none, the request folder already has it (a repeat, which keeps the request's own changes), or the clone was refused, as for a build dir that is a symlink or sits under one, or when no zone folder of the zone is found. A dir not shared never fails `mr folder request`. `--json` adds `build_output`, one `{dir, outcome, why}` per build dir. Zone and attempt folders are made as before.
- **`mr folder refresh <zone folder path>`** sets a zone folder's paths to the zone's paths and root files in the config and brings the folder to the zone branch's tip, so a zone's tests can run between publishes. A repeat changes nothing. It refuses a request or attempt folder (step: `mr join`), an edit that the move would overwrite (the edit is kept), and a zone the config no longer names (step: `mr init --replace`).

## 0.2.0

### Landing

- **`mr rereview` says when the patch is the same.** When the new squash carries the same patch as the reviewed one (the same tree and parents, or equal patch IDs against their first parents), the outcome is `same-patch` and `--json` carries `"same_patch": true`. Its next step leaves it to the caller to decide whether the review still holds on the new base. A squash whose patch changed is answered with only what changed, outcome `rereviewed`, `same_patch` false.

### Upkeep

- **`mr disk` says which folders hold work.** Under each folder's size it shows the folder's unsaved tracked and untracked files, and whether its request has joined its zone: yes, no, or unknown with the reason. `--json` adds `holds`, one entry per folder with `tracked`, `untracked`, `joined` (true, false or null) and `no_joined_answer`. A folder whose joined answer cannot be read is shown as unknown and does not stop the report, and `mr disk` still removes nothing.

### Refusals

- **`mr join` refuses a folder whose HEAD is detached, or whose branch has no commit yet, with the step that fixes it** (exit 1), and so do `mr resolve` and `mr need` for a branch with no commit yet. They used to fail (exit 5) and ask for a repeat that could not succeed. In an attempt folder whose branch has no commit yet, `mr status` names the attempt's branch as its step.
- `mr status` and `mr need` give a failed read of a request's branch in Mortar's own words.

### Fixes

- **`mr save` with a formatter snapshots the folder once,** as `mr join` does. An edit made outside Mortar between the formatter's snapshot and the save is no longer folded into the save.
- **`mr guard install` writes its hooks and `guard.json` through a temporary file it creates fresh** under a drawn name, so a link planted, or a file left by a crash, at a temporary name is never written through or reused.

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
- Needs `git` 2.46 or newer: the guard reads symbolic-ref writes from git's reference-transaction hook, which older git does not pass to it.
- Licensed under the MIT License; see [`LICENSE`](LICENSE).
- The guard stops accidental moves of the protected branches. It is not a defence against someone who can write to the repository's git folder. See [`SECURITY.md`](SECURITY.md).
- `mr disk` and `mr sweep --idle-days` measure and clear the folder named `target` only, whatever `--build-dir` names.
- A relative `core.hooksPath` is read from the top of each worktree, so the guard does not cover a worktree whose path leads to a folder without the hooks.
