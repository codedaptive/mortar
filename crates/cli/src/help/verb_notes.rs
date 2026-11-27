//! verb_notes: the lines a verb's help adds after its row's line, for what its usage cannot say.

/// folder's refresh: what it does to a zone folder's paths (the zone's and the root files) and tip, and why (a
/// zone's tests between publishes); folder's request --with: the merge it cuts on, its refusal on a conflict, and
/// what mr join then gives the zone. squash's --message: the text leads the message and carries no trailers, and the
/// two message shapes land's read_squash reads back wrong (squash_message_requests' known limit). publish's
/// --no-push: what push off leaves alone and the JSON pushed it gives. would-merge's --with: each further squash
/// merged onto the one before, in order, as successive lands would. commit-files: the folders it commits in, the
/// branch and lock in each, its refusal while a zone folder's index holds staged changes except a stopped run's own
/// paths (same, next git restore --staged), and the folders that refuse it; its --message, the message without one,
/// and the folder's other changes left alone. Every other verb has none.
pub fn verb_notes(name: &str) -> &'static [&'static str] {
    match name {
        "folder" => &[
            "refresh <zone folder path>: sets the zone folder's paths to the zone's paths and root files in the \
             config and brings the folder to the zone branch's tip, so a zone's tests can run between publishes",
            "request --with <zone>: cuts the request on a merge of its zone's tip with each partner zone's tip, \
             in the order given; a merge that conflicts is refused, naming the partner and the files; mr join then \
             gives the zone only the request's own commits",
        ],
        "squash" => &[
            "--message <text>: your subject and body lead the squash's message; Mortar adds its request lines and \
             trailers after it, so the text carries no trailers",
            "two shapes read back wrong: a last paragraph of only \"- x: y\" lines is read as request lines, and a \
             subject shaped \"<x>: <n> requests\" is counted as Mortar's",
        ],
        "publish" => &[
            "--no-push: the local origin branch only; nothing is pushed, the remote is not read and nothing is \
             fetched; JSON pushed is true only when an earlier publish of this landing already pushed it",
        ],
        "would-merge" => &[
            "--with <commit or change ID>: each further squash, in the order given, merges onto the result of the \
             one before, as successive lands would; a conflict names its step (1 is the first squash) and its files; \
             no ref moves",
        ],
        "commit-files" => &[
            "in a zone folder at its zone's tip: one commit of the named paths on zone/<zone>, under join-<zone>, \
             refused while the folder's index holds staged changes, except a stopped run's own paths, which answer \
             same with next git restore --staged (unstaged edits are left alone); in the integration folder after \
             mr land: one commit on integration inside the landing, under land; refused in a request or attempt \
             folder (mr save)",
            "--message <text>: the commit's message; without it, the message names the paths; the folder's other \
             changes are left as they are",
        ],
        _ => &[],
    }
}
