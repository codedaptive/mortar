//! CLI bisect/backup/restore-backup joined to real Land, Oplog and Git interfaces.
//! Every test owns and removes its repositories, bare remote and probe files. No live service or team repository.
pub(super) mod fixture;
mod bisect;
mod backup;
mod guards;
