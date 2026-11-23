//! verb_notes: the lines a verb's help adds after its row's line, for what its usage cannot say.

/// folder's refresh: what it does to a zone folder's paths (the zone's and the root files) and tip, and why (a
/// zone's tests between publishes). squash's --message: the text leads the message and carries no trailers, and the
/// two message shapes land's read_squash reads back wrong (squash_message_requests' known limit). publish's
/// --no-push: what push off leaves alone and the JSON pushed it gives. Every other verb has none.
pub fn verb_notes(name: &str) -> &'static [&'static str] {
    match name {
        "folder" => &[
            "refresh <zone folder path>: sets the zone folder's paths to the zone's paths and root files in the \
             config and brings the folder to the zone branch's tip, so a zone's tests can run between publishes",
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
        _ => &[],
    }
}
