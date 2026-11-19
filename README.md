# Mortar

Mortar writes version-control history for teams of coding agents. It makes the folders agents work in, saves their work, joins a request to its zone, squashes a zone's work, lands it, and guards the branches that hold proven code.

The repository stays plain git throughout: any git tool keeps working, and every folder is an ordinary git worktree. Mortar runs `git` as a program and links no git library.

The command is `mr`; `mortar` is the long name for the same program. An agent needs six verbs and never types a git write command.

## Who it is for

- **A harness, or a person, that runs several coding agents in one repository.** Mortar gives each agent a folder of its own, holding only the paths its request needs, and gives the harness the verbs that move finished work toward the main branch: squash, land, publish.
- **A coding agent working in a folder Mortar made.** It uses six verbs. Each one prints where things stand and ends with `next:`, the one step to take.
- **An AI deciding whether Mortar helps it.** Read [`AI_START_HERE.md`](AI_START_HERE.md); the index for machine readers is [`llms.txt`](llms.txt).

## What it is not

- **Not a replacement for git.** It keeps no store of its own. Branches, commits and worktrees are git's, and a person can read all of them with plain git.
- **Not a harness.** It starts no agents, assigns no work and sends no messages. The caller does that.
- **Not a test runner or a reviewer.** It runs no tests of its own and does not decide what may land. The caller decides, and Mortar moves the commits. (`mr bisect` runs the test command you hand it.)
- **Not a hosting or pull-request tool.** The only network use is git's own fetch, push and ls-remote against the one remote you configure.
- **Not a security boundary.** The guard stops accidental moves of the protected branches. It is not a defence against someone who sets out to work around it. See [`SECURITY.md`](SECURITY.md).
- **Not for Windows.** Mortar uses Unix-only system calls.

## Install

Mortar is built from source. It needs a Rust toolchain with Cargo, and `git` 2.46 or newer on the path.

```
git clone https://github.com/codedaptive/mortar.git
cd mortar
cargo build --release --locked -p cli
./target/release/mortar install
```

`mortar install` copies the program to `~/.local/bin/mortar` without root and makes `mr` a link to it. It then offers to add a Mortar skill file to the agent harnesses it finds (Claude Code, Codex, Cursor); that part is optional, and the program works without it.

| Flag | What it does |
|---|---|
| `-y` | Adds the skill to every harness found, with no prompts |
| `--target <harness>[,<harness>...]` | Adds it to the named harnesses only: `claude`, `codex`, `cursor` |
| `--location global\|local` | Puts the skill in your home folder (the default) or in the current project |
| `--no-place` | Leaves the program where it is and does only the rest |
| `--short-name` | Makes the `mr` link even when another program named `mr` comes first on the path |

If another program already owns `mr` on your machine (myrepos does), install leaves it alone and says so. Type `mortar` wherever this page says `mr`.

If `~/.local/bin` is not on your path, install says so; add it. The guard's git hooks run `mortar` by name, so it must be on the path of anything that runs git in a guarded repository.

```
mortar --version      # one line: mortar <version> <commit>
mortar uninstall      # removes exactly what install placed, and nothing else
```

`mortar uninstall` does not touch the config or the hooks that `mr init` and `mr guard install` wrote inside a repository.

## A first run

Try it in a scratch repository. This makes one with a `main` branch, an `integration` branch and a bare remote:

```
mkdir mortar-demo && cd mortar-demo
git init -b main work
cd work
mkdir -p src/a src/b
echo a > src/a/a.txt
echo b > src/b/b.txt
git add -A
git commit -m base
git init --bare -b main ../origin.git
git remote add origin "$(cd .. && pwd)/origin.git"
git push origin main
git branch integration main
```

Set Mortar up, make zone `a` a folder, and make one request folder in it:

```
mr init --actor me --origin main --integration integration --remote origin \
        --zone a=src/a --zone b=src/b
mr folder zone a ../zone-a --actor me
mr folder request a r1 ../r1 --paths src/a --actor me
```

Now work as an agent would, inside the request folder:

```
cd ../r1
mr status                     # ends: next: make your change, then mr save
echo hello > src/a/hello.txt
mr save "add hello"           # ends: next: mr join
mr join                       # ends: next: wait for the zone's tests
```

The request's commit is now on the branch `zone/a`. Look with plain git: `git log zone/a`.

Mortar refuses to make a folder when less than 10 GB of disk is free; `--floor-bytes` on `mr init` sets another floor.

## The six verbs an agent uses

Run inside a request folder Mortar made. They take no folder, zone or request argument: the folder you are in says who you are. Each prints where things stand, with the next step as its last line, and each is safe to repeat.

| Verb | What it does |
|---|---|
| `mr status` | Where the folder is, what has changed, whether the zone has moved, any conflict. It reads only |
| `mr save "<sentence>"` | Keeps the work as a commit, with the one-line sentence as its subject. No staging and no branch names |
| `mr join` | Joins the request to its zone, saving any unsaved work first. If the zone has moved, it rebases the request onto the zone and stops there, so the tests can run again; the next `mr join` joins |
| `mr resolve` | Puts the conflicted files in the folder; run it again once they are fixed |
| `mr need <path>` | Adds a path to the folder. A path outside the request's own paths is recorded as a gap |
| `mr undo` | Takes back the agent's own last save, or the last rebase or resolve step of a join, in its own folder. It refuses a join that has already moved the zone, and any other kind of step, such as `need` |

When a join conflicts:

1. `mr join` exits 2, lists the conflicted files and says that `mr resolve` writes both sides into them with markers. No branch has moved, apart from the save of any unsaved work that `mr join` makes first, and the folder is as it was.
2. `mr resolve` writes each conflicted file into the folder with conflict markers and exits 2. A path it cannot write that way, such as a submodule, is listed instead.
3. Fix the files and remove every marker, then run `mr resolve` again. While markers remain it exits 2 and names the files and lines. Once they are gone it goes on rebasing the request and ends with `next: mr join`; if a later commit of the request conflicts, it stops again with `next: mr resolve`.
4. `mr join`.

## Setting up a repository

These are for the harness or person that runs the team, not for the agents. Every one takes `--actor <name>`. The `mr folder` verbs record it in the operation log with each move they make; `mr init` and `mr guard install` move no ref and record nothing there.

```
mr init --actor me --origin main --integration integration --remote origin \
        --zone a=src/a --zone b=src/b --root-file Cargo.toml
mr guard install --actor me
mr folder zone a ../zone-a --actor me
mr folder request a r1 ../r1 --paths src/a --actor me
```

A zone is a part of the repository one group of agents owns. A request folder holds only the paths its request needs, plus the files at the top of the repository, which every folder has.

`mr init` writes the config and nothing else: it makes no branch and contacts no remote. The origin branch, the integration branch and the remote are yours, and Mortar has no default names for them. Start the integration branch where the origin branch stands (`git branch integration main`). Run `mr init` again with the same values and nothing changes; with other values it is refused unless you add `--replace`.

| `mr init` flag | Meaning |
|---|---|
| `--origin <branch>` | The branch that holds proven code. Required |
| `--integration <branch>` | The branch a landing is tested on before it is published. Required |
| `--remote <name>` | The git remote that `mr publish` and `mr backup` push to. Required |
| `--zone <zone>=<path>[,<path>...]` | A zone and the paths it owns. Repeat for each zone |
| `--root-file <path>` | A path added to every folder Mortar makes. Repeatable |
| `--build-dir <path>` | A build output folder to keep out of saves and snapshots. Repeatable; `target` when none is given. `target`, `node_modules` and `.build` are always kept out |
| `--cap-bytes <n>` | New files larger than this are left out of saves and snapshots, and listed. 5 MiB when not given |
| `--floor-bytes <n>` | Mortar refuses to make a folder when free disk is under this. 10 GB when not given |
| `--formatter <word>` | The formatter to run over changed files at each save, join and snapshot: the program, then one `--formatter` per argument |
| `--formatter-timeout <seconds>` | The limit for one formatter run. Required with `--formatter` |
| `--replace` | Write over a config that holds other values |

More folder verbs:

| Verb | What it does |
|---|---|
| `mr folder zone <zone> <path> [--start <commit>]` | Makes or refills a zone's folder. A zone with no branch yet starts at the origin branch's tip, or at `--start` |
| `mr folder request <zone> <request> <path> --paths <path>...` | Makes a request's folder, cut from the zone's branch |
| `mr folder remove <path>` | Removes a joined and saved request folder, with its branch and its snapshot ref |
| `mr folder attempts <n> <dir>` | Run in a request folder: makes `n` attempt folders of the request under `<dir>`, to try the same work several ways |
| `mr folder keep <k>` | Run in a request folder: keeps attempt `k`, moves the request to it and removes every attempt folder |
| `mr folder drop` | Run in a request folder: removes every attempt and leaves the request as it was |

## Landing

One landing at a time. The origin branch only ever moves forward, and a reviewed squash commit is never rewritten.

| Verb | What it does |
|---|---|
| `mr squash <zone>` | The zone's joined work as one commit on the current origin |
| `mr would-merge <commit or change ID>` | Whether a squash would merge onto integration, without merging |
| `mr rereview <reviewed commit> <new commit>` | Only what changed since a reviewed squash |
| `mr land <commit>` | Moves integration to the squash, or, when integration has moved on since the squash was made, to a merge commit of integration and the squash |
| `mr back-out` | Puts integration back on origin after a failed test |
| `mr publish` | Pushes integration to the remote's origin, then moves the local origin. If someone else moved the remote's origin meanwhile and it does not hold this landing, it fetches that, puts integration back on it and reports it; the squash then lands again |
| `mr after-publish <zone> [--move <zone>]...` | Brings the published zone, and each zone named with `--move`, onto the new origin |

A zone's squash keeps one change ID through every re-squash until it is in origin, so a reviewer can follow one change across several commits. `mr would-merge` takes that change ID or the commit; `mr land` takes the reviewed commit only.

## Other verbs

These are the harness's too. Each takes `--actor <name>` except `mr log`, where `--actor` and `--key` filter the list.

| Verb | What it does |
|---|---|
| `mr log [--ref <full ref name>] [--actor <name>] [--key <operation key>]` | The operation log: every ref move Mortar makes through an operation, with who asked and why. That is every move of a zone, request or attempt branch, of the origin and integration branches, and of a snapshot or squash ref |
| `mr restore <op id> <full ref name>...` | Sets refs back to their values as of an operation in the log, and moves the request folders that stand on them |
| `mr snapshot` | Run in a request or attempt folder: saves the folder's state to a hidden ref, without a commit on its branch |
| `mr overlap <zone>` | Which open requests of a zone change the same paths, before anyone joins |
| `mr disk` | What each folder, the object store and the refs cost on disk, and what a sweep frees |
| `mr sweep [--idle-days <n>] [--ref <full ref name>]...` | Deletes what finished requests and removed zones leave behind (closed requests' snapshot refs, leftover joined branches, replay records), and any ref named with `--ref` that is outside `refs/heads/`, `refs/mortar/` and `refs/remotes/`. With `--idle-days`, also clears the `target` folder of each Mortar folder untouched for that many days |
| `mr backup` | Pushes this machine's in-flight branches, snapshots and squashes to its own namespace on the remote |
| `mr restore-backup <host>` | Brings a machine's backup back from the remote. A local ref that differs from its backup is listed, never moved |
| `mr bisect <last pass> <failed> --timeout <seconds> -- <command>...` | Names the landing that broke a test, running your command at each step in a throwaway worktree |
| `mr guard status` | Whether the guard is in place |

Not in the operation log: the copies a fetch leaves, which `git fetch` writes with no record (`mr publish` fetches the remote's origin branch to `refs/mortar/fetched/<origin branch>`, and `mr restore-backup` fetches backup refs to `refs/mortar-backup/<host>/`), and a folder's own `HEAD`, which is pointed at its branch when the folder is made.

## Output, exit codes and help

Plain output says what happened, then where things stand; the last line of every verb is `next: <one step>`. `--version` prints its one line only, and the guard hook prints nothing when it allows a move. A refusal says why, who owns the thing in the way, and what to do.

Add `--json` to any verb for one JSON object on one line: `verb`, `ok`, `outcome`, `exit`, `next`, `lines`, `owner`, then the verb's own fields. `--version --json` prints `verb`, `ok`, `version` and `commit`; `mortar --json guard-hook` prints nothing when it allows a move.

| Exit code | Meaning |
|---|---|
| 0 | Done, or already done (a repeat of a finished command) |
| 1 | Refused |
| 2 | Conflict: the files are listed |
| 3 | Busy: a lock is held, or a branch or snapshot ref moved while `mr save` or `mr snapshot` ran, so repeat later; or a landing is in flight, so finish it first (`mr publish` or `mr back-out`) |
| 4 | Usage: the command line could not be read |
| 5 | Failure: git or the disk failed. The report says what happened, and the `next:` line says what to do: usually the same command again, sometimes a path to remove by hand first |

A repeat of a finished command exits 0, except `mr folder remove`: the folder is gone, so the repeat is refused. `--key <operation key>` labels a run; without it Mortar derives a label from the verb and the folder. A busy report for a held lock names the label, process and machine of the run that holds it.

`mr help` lists every verb with one line each. `mr help <verb>` and `mr <verb> --help` show one verb's usage. Help reads no repository and changes nothing.

## The guard

After `mr guard install`, only Mortar moves the integration and origin branches. A direct `git update-ref`, `git commit` or `git push` against them is refused. Every move Mortar makes of them names the value it expects to replace, so two movers cannot overwrite each other.

The guard is two git hooks, `reference-transaction` and `pre-push`, in the repository's hooks folder. Mortar never overwrites a hook it did not write: it refuses and names the file. Leave `core.hooksPath` unset or make it an absolute path: a relative one is read from the top of each worktree, so a worktree whose path leads to a folder without the hooks is not guarded.

If Mortar is killed in the middle of a ref move, the log says what happened, and the next command that takes a lock (every verb that moves a ref does) settles it before doing anything else, so repeating the command finishes it once. A folder Mortar was killed while making may be refused on the repeat, with the reason.

## What Mortar keeps in the repository

Mortar's own state lives under git's folders and in git refs, not among your working files.

| Where | What |
|---|---|
| `<git dir>/mortar/config.json` | The config `mr init` writes |
| `<git dir>/mortar/oplog.jsonl` | The operation log |
| `<git dir>/mortar/locks/` | Locks: one per zone for joins, one for landing, one for folder work |
| `<git dir>/mortar/guard.json`, `<git dir>/mortar/landing.json` | The guard's branches; the landing in flight |
| `mortar-folder.json`, `mortar-join.json` in a worktree's own git dir | What a folder is; a join paused on a conflict |
| `zone/<zone>`, `req/<zone>/<request>`, `try/<zone>/<request>/<k>` | The zone, request and attempt branches |
| `refs/mortar/snap/`, `refs/mortar/trysnap/`, `refs/mortar/squash/` | Snapshots of request and attempt folders; each zone's squash |

`<git dir>` is the repository's shared git folder (`git rev-parse --git-common-dir`). The commits Mortar writes carry `Mortar-` trailers (`Mortar-Change-Id`, `Mortar-Request`, `Mortar-Zone`, and `Mortar-Squash-Of` on a squash) that say which change, request and zone each belongs to.

## With Tessera and Temperare

Tessera reads the code, Mortar writes its history, and Temperare checks it. Each one works alone.

- **Tessera** (`ts`) answers questions about a codebase from an index: where a symbol is defined, what calls it, what a change would reach.
- **Temperare** (`tm`) checks code against the rules a repository has chosen to enforce, and reports what it finds.

Mortar needs neither and calls neither. It holds no opinion about the code it moves, beyond running the formatter you configure over the changed files when it saves, joins and snapshots.

## Status

Version 0.1.0, the first public version. See [`CHANGELOG.md`](CHANGELOG.md).

- Mortar is built from source. The crates are not published to crates.io.
- It runs on Unix systems only.
- The command line, the `--json` fields and the layout under `refs/mortar/` may still change before 1.0.

## License

MIT. See [`LICENSE`](LICENSE).

## Working on Mortar

[`CONTRIBUTING.md`](CONTRIBUTING.md) says how to build, test and send a change. [`docs/README.md`](docs/README.md) is the map of the documents and the code. To report a vulnerability, see [`SECURITY.md`](SECURITY.md).
