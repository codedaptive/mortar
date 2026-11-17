//! WO-cli-rit-6: real CLI/Land/Folder/Oplog/Git interfaces (S-135, S-136, S-137).
//! Each case owns and removes its repository, worktrees and bare remote. No live service or team repository.
mod fixture;
mod would_merge;
mod landing;
mod stopped;
mod killed;
