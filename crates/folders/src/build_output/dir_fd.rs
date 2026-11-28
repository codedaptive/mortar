//! DirFd on macOS: a directory held open by its descriptor, and every call into libSystem the build-output clone makes.
//! It holds the crate's only `unsafe`: std opens nothing relative to a directory descriptor and has no mkdirat or
//! fclonefileat, and libc would be a new dependency, so this file declares openat(2), faccessat(2), mkdirat(2) and
//! fclonefileat(2) itself. DirFd's methods share one impl in this one file, an exception to one impl block per file, so
//! that every `unsafe` call and its declaration sit together, where one reading checks them all; that is why the file
//! runs past 150 lines counting comments (under it in code lines alone).
//! Directories are opened search-only (O_SEARCH), so one with execute but no read permission is walked; the one
//! descriptor that must be readable is a clone's source, since fclonefileat answers EBADF from a search-only one.
//! No unit test (no sibling _tests.rs): it only opens, makes and clones on the filesystem;
//! tests/folders/build_output.rs covers it through clone_build_dirs. Other targets use dir_path.rs instead.

use std::ffi::{c_int, CStr, CString, OsStr};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
use std::path::Path;

use super::Entry;

/// macOS's values and declarations, from the headers each group names.
mod sys {
    use std::ffi::{c_char, c_int};

    // <sys/fcntl.h>
    pub(super) const O_RDONLY: c_int = 0;
    pub(super) const O_NONBLOCK: c_int = 0x4;
    pub(super) const O_NOFOLLOW: c_int = 0x100;
    pub(super) const O_DIRECTORY: c_int = 0x10_0000;
    pub(super) const O_CLOEXEC: c_int = 0x100_0000;
    pub(super) const AT_SYMLINK_NOFOLLOW: c_int = 0x20;
    // <sys/fcntl.h>:163-164: O_EXEC 0x40000000, O_SEARCH (O_EXEC | O_DIRECTORY).
    pub(super) const O_SEARCH: c_int = 0x4000_0000 | O_DIRECTORY;
    // <sys/unistd.h>
    pub(super) const F_OK: c_int = 0;
    // <sys/errno.h>
    pub(super) const ENOENT: i32 = 2;
    pub(super) const EEXIST: i32 = 17;
    pub(super) const ENOTDIR: i32 = 20;
    pub(super) const ELOOP: i32 = 62;

    // openat is variadic in C (the mode follows only with O_CREAT); mode_t is a 16-bit unsigned int on macOS;
    // faccessat is declared in <sys/unistd.h>, fclonefileat in <sys/clonefile.h>.
    extern "C" {
        pub(super) fn openat(fd: c_int, path: *const c_char, oflag: c_int, ...) -> c_int;
        pub(super) fn faccessat(fd: c_int, path: *const c_char, mode: c_int, flag: c_int) -> c_int;
        pub(super) fn mkdirat(fd: c_int, path: *const c_char, mode: u16) -> c_int;
        pub(super) fn fclonefileat(src_fd: c_int, dst_dirfd: c_int, dst: *const c_char, flags: u32) -> c_int;
    }
}

/// A directory open search-only by descriptor, closed on drop; in Entry::File, another non-directory file open for
/// reading; from `readable`, a directory open for reading. Each name its methods take is one path component, looked
/// up inside this directory, so nothing they do resolves a path from the top again.
pub(crate) struct DirFd(OwnedFd);

impl DirFd {
    /// None: clonefile(2) exists here, so nothing refuses a clone before it is tried.
    pub(crate) const NO_CLONE: Option<&'static str> = None;

    /// Opens `path`, a trusted root (the caller's folder), search-only, following symlinks on the way there: on
    /// macOS `/tmp` itself is one. std opens it close-on-exec.
    pub(crate) fn open(path: &Path) -> io::Result<DirFd> {
        std::fs::OpenOptions::new().read(true).custom_flags(sys::O_SEARCH).open(path).map(|file| DirFd(file.into()))
    }

    /// What `name` is in this directory, found without following a symlink there (Entry). A directory is answered
    /// open search-only and any other file that is not a symlink open for reading, so a clone can be made from the
    /// very file this look opened.
    pub(crate) fn entry(&self, name: &OsStr) -> Entry {
        // No file name holds a NUL byte, so a name that does is missing.
        let Ok(c_name) = CString::new(name.as_bytes()) else {
            return Entry::Missing;
        };
        let error = match self.open_at(&c_name, sys::O_SEARCH) {
            Ok(fd) => return Entry::Dir(DirFd(fd)),
            Err(error) => error,
        };
        match error.raw_os_error() {
            Some(sys::ELOOP) => Entry::Symlink,
            Some(sys::ENOENT) => Entry::Missing,
            // Not a directory, or a symlink refused that way: opened again without O_DIRECTORY a symlink answers
            // ELOOP, and a file that cannot be opened for reading is Unreadable. O_NONBLOCK: a fifo cannot hang the
            // open.
            Some(sys::ENOTDIR) => match self.open_at(&c_name, sys::O_RDONLY | sys::O_NONBLOCK) {
                Ok(fd) => Entry::File(DirFd(fd)),
                Err(again) if again.raw_os_error() == Some(sys::ELOOP) => Entry::Symlink,
                Err(again) => Entry::Unreadable(again.to_string()),
            },
            // The open failed some other way (any non-directory answered ENOTDIR above): faccessat (existence only,
            // the link itself) tells a directory that is there but cannot be opened from a name that cannot be looked
            // up at all.
            _ => {
                // SAFETY: `c_name` is a valid NUL-terminated string that outlives the call, the descriptor is open
                // while `self` lives, and faccessat only reads the string.
                let seen = unsafe {
                    sys::faccessat(self.0.as_raw_fd(), c_name.as_ptr(), sys::F_OK, sys::AT_SYMLINK_NOFOLLOW)
                };
                match seen {
                    0 => Entry::Other(error.to_string()),
                    _ => Entry::Unseen(io::Error::last_os_error().to_string()),
                }
            }
        }
    }

    /// Makes the directory `name` here with mkdirat(2), mode 0o777 less the umask as create_dir makes it. One that
    /// already exists is not an error: the caller opens `name` next without following a symlink, and that decides.
    pub(crate) fn make_dir(&self, name: &OsStr) -> Result<(), String> {
        let c_name = c_name(name)?;
        // SAFETY: `c_name` is a valid NUL-terminated string that outlives the call, the descriptor is open while
        // `self` lives, and mkdirat only reads the string.
        if unsafe { sys::mkdirat(self.0.as_raw_fd(), c_name.as_ptr(), 0o777) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(sys::EEXIST) => Ok(()),
            _ => Err(error.to_string()),
        }
    }

    /// A second descriptor for this directory, so a walk can own the one it starts from.
    pub(crate) fn try_clone(&self) -> Result<DirFd, String> {
        self.0.try_clone().map(DirFd).map_err(|e| e.to_string())
    }

    /// This directory opened again for reading, through "." so no name is looked up: a clone's source must be
    /// readable. The error is the OS error alone.
    pub(crate) fn readable(&self) -> Result<DirFd, String> {
        self.open_at(c".", sys::O_RDONLY | sys::O_DIRECTORY).map(DirFd).map_err(|e| e.to_string())
    }

    /// Clones what this descriptor holds (a directory tree or a file) into the directory `target` as `name`, with
    /// fclonefileat(2): the clone shares the blocks until a file in either changes. The source is this open
    /// descriptor, which must be readable (Entry::File's, or one from `readable`), so no name is looked up on its
    /// side; `target` may be search-only, but its directory must be writable. Never falls back to a byte copy: on any
    /// failure nothing is copied and the error says why.
    pub(crate) fn clone_into(&self, target: &DirFd, name: &OsStr) -> Result<(), String> {
        let c_name = c_name(name).map_err(|why| format!("{why}; nothing copied"))?;
        // SAFETY: `c_name` is a valid NUL-terminated string that outlives the call, both descriptors are open while
        // `self` and `target` live, and fclonefileat only reads the string.
        let status = unsafe { sys::fclonefileat(self.0.as_raw_fd(), target.0.as_raw_fd(), c_name.as_ptr(), 0) };
        if status != 0 {
            return Err(format!("clonefile refused: {}", io::Error::last_os_error()));
        }
        Ok(())
    }

    /// openat(2) of `name` here with `flags` (the access mode among them), never following a symlink, close-on-exec.
    fn open_at(&self, name: &CStr, flags: c_int) -> io::Result<OwnedFd> {
        let oflag = sys::O_NOFOLLOW | sys::O_CLOEXEC | flags;
        // SAFETY: `name` is a valid NUL-terminated string that outlives the call, the descriptor is open while `self`
        // lives, and openat only reads the string; no mode is passed, which openat reads only with O_CREAT.
        let fd = unsafe { sys::openat(self.0.as_raw_fd(), name.as_ptr(), oflag) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openat returned a new descriptor that nothing else owns, so the OwnedFd may close it.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

/// `name` as a C string; a NUL byte in it is the error.
fn c_name(name: &OsStr) -> Result<CString, String> {
    CString::new(name.as_bytes()).map_err(|_| format!("\"{}\" holds a NUL byte", Path::new(name).display()))
}
