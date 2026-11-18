# Mortar

Mortar writes version-control history for teams of coding agents. It makes the folders agents work in, saves their work, joins a request to its zone, squashes a zone's work, lands it, and guards the branches that hold proven code. The command is `mr`; `mortar` is a second name for the same program.

The repository stays plain git throughout: any git tool keeps working, and every folder is an ordinary git worktree. Mortar runs `git` as a program and links no git library.

An agent needs six verbs and never types a git write command.

## Install

```
cargo install --path crates/cli --locked
```

This installs both names, `mr` and `mortar`. Mortar needs `git` on the path.

## The six verbs an agent uses

Run inside a folder Mortar made. Each prints where things stand, with the next step as its last line, and each is safe to repeat.

| Verb | What it does |
|---|---|
| `mr status` | Where the folder is, what has changed, whether the zone has moved, any conflict |
| `mr save "<sentence>"` | Keeps the work as a commit. No staging and no branch names |
| `mr join` | Joins the request to its zone, rebasing first if the zone has moved |
| `mr resolve` | Puts the conflicted files in the folder; run it again once they are fixed |
| `mr need <path>` | Adds a path to the folder |
| `mr undo` | Takes back the agent's own last save, or the last rebase or resolve step of a join, in its own folder. It refuses a join that has already moved the zone, and any other kind of step, such as `need` |

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

## Landing

One landing at a time; proven commits are never rewritten.

| Verb | What it does |
|---|---|
| `mr squash <zone>` | The zone's joined work as one commit on the current origin |
| `mr would-merge <change>` | Whether a squash would merge onto integration, without merging |
| `mr rereview <first> <second>` | Only what changed since a reviewed commit |
| `mr land <commit>` | Moves integration to the squash, or, when integration has moved on since the squash was made, to a merge commit of integration and the squash |
| `mr back-out` | Puts integration back on origin after a failed test |
| `mr publish` | Pushes integration to the remote's origin, then moves the local origin. If someone else moved the remote's origin meanwhile and it does not hold this landing, it fetches that, puts integration back on it and reports it; the squash then lands again |
| `mr after-publish <zone>` | Replays a zone onto the new origin |

Also: `mr log` (every ref move Mortar makes, who asked and why), `mr disk` and `mr sweep` (what folders and the store cost, and what can be freed), `mr overlap`, `mr snapshot`, `mr restore`, `mr backup`, `mr restore-backup`, `mr bisect`.

Add `--json` to any of these verbs for one JSON object with the outcome, the exit code and the next step.

## The guard

After `mr guard install`, only Mortar moves the integration and origin branches. A direct `git update-ref`, `git commit` or `git push` against them is refused. Every move Mortar makes names the value it expects to replace, so two movers cannot overwrite each other.

The guard lives in the repository's hooks folder. Leave `core.hooksPath` unset or make it an absolute path: a relative one is read from the top of each worktree, so a worktree whose path leads to a folder without the hooks is not guarded.

If Mortar is killed in the middle of a ref move, the log says what happened and the next run settles it before doing anything else, so repeating the command finishes it once. A folder Mortar was killed while making may be refused on the repeat, with the reason.

## With Tessera and Temperare

Tessera reads the code, Mortar writes its history, and Temperare checks it. Each one works alone.

Mortar holds no opinion about the code it moves, beyond running the formatter you configure over the files when it saves and joins. It does not run tests of its own, review, or decide what may land: the caller decides, and Mortar moves the commits. (`mr bisect` runs the test command you hand it.)

## Working on Mortar

`CLAUDE.md` is where an agent starts. `make accept` runs the acceptance list on throwaway repositories.
