//! Real CLI/Land/Folder/Oplog/Git interfaces.
//! Each case owns and removes its repository, worktrees and bare remote. No live service or team repository.
mod fixture;
mod would_merge;
mod landing;
mod stopped;
mod killed;
