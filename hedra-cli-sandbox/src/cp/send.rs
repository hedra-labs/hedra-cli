//! Copy in: builds the tar stream from local files.
//!
//! The walk opens every entry relative to its parent's handle and checks it
//! is still what the walk saw, so a file swapped for a symlink or a FIFO
//! during the copy fails it instead of sending what the link points to.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bytes::Bytes;
use tar::{Builder, EntryType, Header};
use tokio::sync::mpsc;

use super::names::escape_controls;
use super::{Progress, CHUNK, TAR_RECORD};

/// What the tar builder hands the sending task.
pub(crate) enum Up {
    Data(Bytes),
    /// The whole stream, padded, has been handed over.
    Done,
}

/// Entries the walk left out, by their entry name, and the warnings to print.
#[derive(Default)]
pub(crate) struct Walked {
    pub(crate) skipped: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

/// Writes `src` as a tar stream rooted at `name`, padded to a whole record,
/// into `tx`, then sends [`Up::Done`]. The error names the file that failed.
pub(crate) fn send_tar(
    src: &Path,
    name: &str,
    tx: mpsc::Sender<Up>,
    progress: Arc<Progress>,
) -> Result<Walked, String> {
    let mut w = ChunkWriter {
        tx,
        buf: Vec::with_capacity(CHUNK),
        total: 0,
        progress: progress.clone(),
        poisoned: false,
    };
    let mut b = Builder::new(&mut w);
    let mut walked = Walked::default();
    if let Err(e) = walk(&mut b, src, name, &progress, &mut walked) {
        // Nothing more reaches the call: the builder's drop would write the
        // end blocks, and a whole chunk of them could be sent before the
        // reset, so the remote tar would see a clean end.
        b.get_mut().poisoned = true;
        std::mem::forget(b);
        return Err(e);
    }
    // Past the walk only the call can fail, and the call says why.
    let _ = finish(b);
    Ok(walked)
}

fn finish(b: Builder<&mut ChunkWriter>) -> io::Result<()> {
    let w = b.into_inner()?;
    w.pad()?;
    w.flush()?;
    w.tx.blocking_send(Up::Done)
        .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))
}

/// Hands what is written to it to the sending task in `CHUNK`-byte pieces.
struct ChunkWriter {
    tx: mpsc::Sender<Up>,
    buf: Vec<u8>,
    total: u64,
    progress: Arc<Progress>,
    /// Set when the walk failed; every later write is discarded.
    poisoned: bool,
}

impl ChunkWriter {
    fn hand_over(&mut self, chunk: Vec<u8>) -> io::Result<()> {
        if self.poisoned {
            return Ok(());
        }
        let n = chunk.len();
        self.tx
            .blocking_send(Up::Data(Bytes::from(chunk)))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "the call ended"))?;
        self.progress.add(n as u64);
        Ok(())
    }

    /// Pads the stream to a whole tar record, as GNU tar pads its own.
    fn pad(&mut self) -> io::Result<()> {
        let rem = self.total % TAR_RECORD;
        if rem != 0 {
            self.write_all(&vec![0; (TAR_RECORD - rem) as usize])?;
        }
        Ok(())
    }
}

impl Write for ChunkWriter {
    fn write(&mut self, p: &[u8]) -> io::Result<usize> {
        if self.poisoned {
            return Ok(p.len());
        }
        self.buf.extend_from_slice(p);
        self.total += p.len() as u64;
        while self.buf.len() >= CHUNK {
            let rest = self.buf.split_off(CHUNK);
            let chunk = std::mem::replace(&mut self.buf, rest);
            self.hand_over(chunk)?;
        }
        Ok(p.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buf.is_empty() {
            let chunk = std::mem::take(&mut self.buf);
            self.hand_over(chunk)?;
        }
        Ok(())
    }
}

/// Yields exactly `left` bytes of a file: a file that grew is cut there, and
/// one that ends sooner fails with `UnexpectedEof` naming it.
struct Exact<'a> {
    file: std::fs::File,
    left: u64,
    size: u64,
    shown: &'a Path,
}

impl Read for Exact<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.left == 0 || buf.is_empty() {
            return Ok(0);
        }
        let want = buf
            .len()
            .min(usize::try_from(self.left).unwrap_or(usize::MAX));
        let n = self.file.read(&mut buf[..want])?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "{} ended before the {} bytes it held when the copy began",
                    self.shown.display(),
                    self.size
                ),
            ));
        }
        self.left -= n as u64;
        Ok(n)
    }
}

fn header(kind: EntryType, mode: u32, mtime: i64, size: u64) -> Header {
    let mut h = Header::new_gnu();
    h.set_entry_type(kind);
    h.set_mode(mode & 0o7777);
    h.set_uid(0);
    h.set_gid(0);
    h.set_mtime(mtime.max(0) as u64);
    h.set_size(size);
    h
}

fn read_failed(shown: &Path, e: impl std::fmt::Display) -> String {
    format!("cannot read {}: {e}", shown.display())
}

fn changed(shown: &Path) -> String {
    format!(
        "{} changed while it was being copied; nothing was sent for it",
        shown.display()
    )
}

/// Appends a regular file's body, mapping a short read to its own message.
fn append_file<W: Write>(
    b: &mut Builder<W>,
    entry: &Path,
    mut h: Header,
    file: std::fs::File,
    size: u64,
    shown: &Path,
) -> Result<(), String> {
    let r = Exact {
        file,
        left: size,
        size,
        shown,
    };
    b.append_data(&mut h, entry, r).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            e.to_string()
        } else {
            read_failed(shown, e)
        }
    })
}

#[cfg_attr(windows, allow(dead_code))]
fn left_out(walked: &mut Walked, entry: &Path) {
    let shown = escape_controls(&entry.to_string_lossy());
    walked.warnings.push(format!(
        "left out {shown}: not a file, a directory or a symlink"
    ));
    walked.skipped.push(shown);
}

#[cfg(unix)]
fn walk<W: Write>(
    b: &mut Builder<W>,
    src: &Path,
    name: &str,
    progress: &Progress,
    walked: &mut Walked,
) -> Result<(), String> {
    use rustix::fs::{openat, statat, AtFlags, Mode, OFlags, CWD};

    let parent = src.parent().unwrap_or(Path::new("/"));
    let base = src.file_name().ok_or_else(|| changed(src))?;
    let pfd = openat(
        CWD,
        parent,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| read_failed(parent, io::Error::from(e)))?;
    let seen = statat(&pfd, base, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|e| read_failed(src, io::Error::from(e)))?;
    unix::entry(
        b,
        &pfd,
        base,
        &seen,
        src,
        Path::new(name),
        true,
        progress,
        walked,
    )
}

#[cfg(unix)]
mod unix {
    use super::*;
    use rustix::fs::{fstat, openat, readlinkat, statat, AtFlags, FileType, Mode, OFlags, Stat};
    use std::ffi::OsStr;
    use std::os::fd::OwnedFd;

    // `st_mtime` is narrower than i64 on some targets.
    #[allow(clippy::useless_conversion)]
    fn mtime_of(st: &Stat) -> i64 {
        i64::from(st.st_mtime)
    }

    // `st_mode` is u16 on some targets.
    #[allow(clippy::useless_conversion)]
    fn mode_of(st: &Stat) -> u32 {
        u32::from(st.st_mode)
    }

    fn kind(st: &Stat) -> FileType {
        FileType::from_raw_mode(st.st_mode as _)
    }

    /// Opens `name` in `dirfd` without following a link or blocking on a
    /// FIFO, and checks it is the file or directory the walk saw.
    fn open_seen(
        dirfd: &OwnedFd,
        name: &OsStr,
        seen: &Stat,
        shown: &Path,
    ) -> Result<(OwnedFd, Stat), String> {
        let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        if kind(seen) == FileType::Directory {
            flags |= OFlags::DIRECTORY;
        }
        let fd = openat(dirfd, name, flags, Mode::empty()).map_err(|e| {
            if e == rustix::io::Errno::LOOP || e == rustix::io::Errno::NOTDIR {
                changed(shown)
            } else {
                read_failed(shown, io::Error::from(e))
            }
        })?;
        let st = fstat(&fd).map_err(|e| read_failed(shown, io::Error::from(e)))?;
        if kind(&st) != kind(seen) || st.st_dev != seen.st_dev || st.st_ino != seen.st_ino {
            return Err(changed(shown));
        }
        Ok((fd, st))
    }

    /// Appends one entry the walk saw in `dirfd`, and everything under it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn entry<W: Write>(
        b: &mut Builder<W>,
        dirfd: &OwnedFd,
        name: &OsStr,
        seen: &Stat,
        shown: &Path,
        entry: &Path,
        top: bool,
        progress: &Progress,
        walked: &mut Walked,
    ) -> Result<(), String> {
        let mtime = mtime_of(seen);
        match kind(seen) {
            FileType::RegularFile => {
                let (fd, st) = open_seen(dirfd, name, seen, shown)?;
                progress.began(&entry.to_string_lossy());
                let size = st.st_size as u64;
                let h = header(EntryType::Regular, mode_of(&st), mtime_of(&st), size);
                append_file(b, entry, h, std::fs::File::from(fd), size, shown)
            }
            FileType::Directory => {
                let (fd, st) = open_seen(dirfd, name, seen, shown)?;
                // A tree merged into `.` has no entry for its top, so the
                // directory it lands in keeps its mode and mtime.
                if entry != Path::new(".") {
                    progress.began(&entry.to_string_lossy());
                    let mut h = header(EntryType::Directory, mode_of(&st), mtime_of(&st), 0);
                    b.append_data(&mut h, entry, io::empty())
                        .map_err(|e| read_failed(shown, e))?;
                }
                let mut names: Vec<std::ffi::OsString> = Vec::new();
                let dir = rustix::fs::Dir::read_from(&fd)
                    .map_err(|e| read_failed(shown, io::Error::from(e)))?;
                for ent in dir {
                    let ent = ent.map_err(|e| read_failed(shown, io::Error::from(e)))?;
                    let n = ent.file_name().to_bytes();
                    if n == b"." || n == b".." {
                        continue;
                    }
                    use std::os::unix::ffi::OsStrExt;
                    names.push(OsStr::from_bytes(n).to_owned());
                }
                names.sort();
                for child in names {
                    let child_shown = shown.join(&child);
                    let child_entry = if entry == Path::new(".") {
                        PathBuf::from(&child)
                    } else {
                        entry.join(&child)
                    };
                    let st = statat(&fd, child.as_os_str(), AtFlags::SYMLINK_NOFOLLOW)
                        .map_err(|e| read_failed(&child_shown, io::Error::from(e)))?;
                    super::unix::entry(
                        b,
                        &fd,
                        &child,
                        &st,
                        &child_shown,
                        &child_entry,
                        false,
                        progress,
                        walked,
                    )?;
                }
                Ok(())
            }
            FileType::Symlink if !top => {
                let target = readlinkat(dirfd, name, Vec::new())
                    .map_err(|e| read_failed(shown, io::Error::from(e)))?;
                use std::os::unix::ffi::OsStrExt;
                let target = Path::new(OsStr::from_bytes(target.to_bytes())).to_path_buf();
                progress.began(&entry.to_string_lossy());
                let mut h = header(EntryType::Symlink, mode_of(seen), mtime, 0);
                b.append_link(&mut h, entry, &target)
                    .map_err(|e| read_failed(shown, e))
            }
            FileType::Symlink => Err(changed(shown)),
            _ if top => Err(format!("{} is not a file or a directory", shown.display())),
            _ => {
                left_out(walked, entry);
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
fn walk<W: Write>(
    b: &mut Builder<W>,
    src: &Path,
    name: &str,
    progress: &Progress,
    walked: &mut Walked,
) -> Result<(), String> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;

    let parent = src.parent().unwrap_or(Path::new("\\"));
    let base = src.file_name().ok_or_else(|| changed(src))?;
    let pdir = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(parent)
        .map_err(|e| read_failed(parent, e))?;
    let meta = std::fs::symlink_metadata(src).map_err(|e| read_failed(src, e))?;
    let seen = if meta.file_type().is_symlink() {
        windows::Kind::Link
    } else if meta.is_dir() {
        windows::Kind::Dir
    } else if meta.is_file() {
        windows::Kind::File
    } else {
        return Err(format!("{} is not a file or a directory", src.display()));
    };
    windows::entry(
        b,
        &pdir,
        base,
        seen,
        src,
        Path::new(name),
        true,
        progress,
        walked,
    )
}

const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;
const SYMLINK_FLAG_RELATIVE: u32 = 1;

/// The target in a symlink's or junction's `REPARSE_DATA_BUFFER`, as
/// `std::fs::read_link` gives it: an absolute target's `\??\` prefix becomes
/// a drive or UNC path where it names one, else a verbatim `\\?\` path.
#[cfg_attr(not(windows), allow(dead_code))]
fn reparse_target(b: &[u8]) -> Option<String> {
    let u16_at = |at: usize| Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?));
    let tag = u32::from_le_bytes(b.get(0..4)?.try_into().ok()?);
    let (paths, relative) = match tag {
        IO_REPARSE_TAG_SYMLINK => {
            let flags = u32::from_le_bytes(b.get(16..20)?.try_into().ok()?);
            (20, flags & SYMLINK_FLAG_RELATIVE != 0)
        }
        IO_REPARSE_TAG_MOUNT_POINT => (16, false),
        _ => return None,
    };
    let off = paths + usize::from(u16_at(8)?);
    let len = usize::from(u16_at(10)?);
    let (pairs, _) = b.get(off..off.checked_add(len)?)?.as_chunks::<2>();
    let wide: Vec<u16> = pairs.iter().map(|c| u16::from_le_bytes(*c)).collect();
    let s = String::from_utf16_lossy(&wide);
    Some(match s.strip_prefix(r"\??\") {
        Some(rest) if !relative => {
            if let Some(unc) = rest.strip_prefix(r"UNC\") {
                format!(r"\\{unc}")
            } else if rest.as_bytes().get(1) == Some(&b':') {
                rest.to_string()
            } else {
                format!(r"\\?\{rest}")
            }
        }
        _ => s,
    })
}

/// The Windows walk: every entry is opened relative to its parent's handle
/// with `NtCreateFile`, and each directory is listed from its own handle.
#[cfg(windows)]
mod windows {
    use super::*;
    use std::ffi::{OsStr, OsString};
    use std::fs::File;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        NtCreateFile, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN,
        FILE_OPEN_FOR_BACKUP_INTENT, FILE_OPEN_REPARSE_POINT, FILE_SYNCHRONOUS_IO_NONALERT,
    };
    use windows_sys::Win32::Foundation::{
        RtlNtStatusToDosError, ERROR_NO_MORE_FILES, HANDLE, OBJ_CASE_INSENSITIVE, UNICODE_STRING,
    };
    use windows_sys::Win32::Storage::FileSystem::MAXIMUM_REPARSE_DATA_BUFFER_SIZE;
    use windows_sys::Win32::Storage::FileSystem::{
        FileAttributeTagInfo, FileBasicInfo, FileIdBothDirectoryInfo, FileIdInfo, FileStandardInfo,
        GetFileInformationByHandle, GetFileInformationByHandleEx, BY_HANDLE_FILE_INFORMATION,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO,
        FILE_BASIC_INFO, FILE_GENERIC_READ, FILE_ID_BOTH_DIR_INFO, FILE_ID_INFO,
        FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FILE_STANDARD_INFO, SYNCHRONIZE,
    };
    use windows_sys::Win32::System::Ioctl::FSCTL_GET_REPARSE_POINT;
    use windows_sys::Win32::System::IO::{DeviceIoControl, IO_STATUS_BLOCK};

    /// What an entry is, as its directory listing or its handle says.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub(super) enum Kind {
        File,
        Dir,
        /// A symlink, junction or mount point: a name surrogate.
        Link,
    }

    /// `IsReparseTagNameSurrogate`.
    fn name_surrogate(tag: u32) -> bool {
        tag & 0x2000_0000 != 0
    }

    fn kind_of(attributes: u32, tag: u32) -> Kind {
        if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 && name_surrogate(tag) {
            Kind::Link
        } else if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Kind::Dir
        } else {
            Kind::File
        }
    }

    /// What [`open_at`] opens.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Open {
        File,
        Dir,
        /// The reparse point itself, file or directory, to read its data.
        Link,
    }

    /// Opens `name` in the directory `parent` with `NtCreateFile`.
    fn open_at(
        parent: &impl AsRawHandle,
        name: &OsStr,
        what: Open,
        reparse_point: bool,
    ) -> io::Result<File> {
        let mut wide: Vec<u16> = name.encode_wide().collect();
        let bytes = u16::try_from(wide.len() * 2)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidFilename))?;
        let us = UNICODE_STRING {
            Length: bytes,
            MaximumLength: bytes,
            Buffer: wide.as_mut_ptr(),
        };
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent.as_raw_handle() as HANDLE,
            ObjectName: &us,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: std::ptr::null(),
            SecurityQualityOfService: std::ptr::null(),
        };
        let (access, kind) = match what {
            Open::Dir => (
                FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT,
            ),
            Open::File => (FILE_GENERIC_READ | SYNCHRONIZE, FILE_NON_DIRECTORY_FILE),
            Open::Link => (
                FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                FILE_OPEN_FOR_BACKUP_INTENT,
            ),
        };
        let mut options = kind | FILE_SYNCHRONOUS_IO_NONALERT;
        if reparse_point {
            options |= FILE_OPEN_REPARSE_POINT;
        }
        let mut handle: HANDLE = std::ptr::null_mut();
        let mut iosb = IO_STATUS_BLOCK::default();
        // SAFETY: every pointer is to a live local, and `wide` outlives the
        // call; a handle returned is owned from here on.
        let status = unsafe {
            NtCreateFile(
                &mut handle,
                access,
                &attrs,
                &mut iosb,
                std::ptr::null(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_OPEN,
                options,
                std::ptr::null(),
                0,
            )
        };
        if status < 0 {
            // SAFETY: maps a status code; no memory is touched.
            let code = unsafe { RtlNtStatusToDosError(status) };
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        // SAFETY: NtCreateFile succeeded, so `handle` is an open handle.
        Ok(File::from(unsafe {
            OwnedHandle::from_raw_handle(handle as _)
        }))
    }

    /// Reads one fixed-size information class from a handle.
    fn info<T: Default>(f: &File, class: i32) -> io::Result<T> {
        let mut v = T::default();
        // SAFETY: `v` is a `T` of the size passed, as the class expects.
        let ok = unsafe {
            GetFileInformationByHandleEx(
                f.as_raw_handle() as HANDLE,
                class,
                (&mut v as *mut T).cast(),
                std::mem::size_of::<T>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(v)
    }

    /// What an open handle is.
    struct Stat {
        kind: Kind,
        reparse_point: bool,
        size: u64,
        /// In Unix seconds.
        mtime: i64,
    }

    fn stat(f: &File) -> io::Result<Stat> {
        let tag: FILE_ATTRIBUTE_TAG_INFO = info(f, FileAttributeTagInfo)?;
        let std_info: FILE_STANDARD_INFO = info(f, FileStandardInfo)?;
        let basic: FILE_BASIC_INFO = info(f, FileBasicInfo)?;
        Ok(Stat {
            kind: kind_of(tag.FileAttributes, tag.ReparseTag),
            reparse_point: tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0,
            size: std_info.EndOfFile.max(0) as u64,
            // 100 ns intervals since 1601.
            mtime: (basic.LastWriteTime / 10_000_000 - 11_644_473_600).max(0),
        })
    }

    /// The file a handle is open on: its volume and file id, from
    /// `FileIdInfo`, or from `GetFileInformationByHandle` where the file
    /// system has no 128-bit ids.
    fn identity(f: &File) -> io::Result<(u64, [u8; 16])> {
        if let Ok(id) = info::<FILE_ID_INFO>(f, FileIdInfo) {
            return Ok((id.VolumeSerialNumber, id.FileId.Identifier));
        }
        // SAFETY: zeroed is a valid value of this plain C struct.
        let mut bh: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: `bh` is writable and of the type the call fills.
        if unsafe { GetFileInformationByHandle(f.as_raw_handle() as HANDLE, &mut bh) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let index = (u64::from(bh.nFileIndexHigh) << 32) | u64::from(bh.nFileIndexLow);
        let mut id = [0u8; 16];
        id[..8].copy_from_slice(&index.to_le_bytes());
        Ok((u64::from(bh.dwVolumeSerialNumber), id))
    }

    /// Opens a file or directory the walk saw, without following a name
    /// surrogate; a reparse point of another kind (a cloud placeholder, a
    /// deduplicated file) is opened again through it, as what it holds,
    /// and must still be the same file.
    fn open_seen(
        parent: &File,
        name: &OsStr,
        seen: Kind,
        shown: &Path,
    ) -> Result<(File, u64, i64), String> {
        let what = if seen == Kind::Dir {
            Open::Dir
        } else {
            Open::File
        };
        let f = open_at(parent, name, what, true).map_err(|e| read_failed(shown, e))?;
        let st = stat(&f).map_err(|e| read_failed(shown, e))?;
        if st.kind != seen {
            return Err(changed(shown));
        }
        if !st.reparse_point {
            return Ok((f, st.size, st.mtime));
        }
        let first = identity(&f).map_err(|e| read_failed(shown, e))?;
        let g = open_at(parent, name, what, false).map_err(|e| read_failed(shown, e))?;
        let st = stat(&g).map_err(|e| read_failed(shown, e))?;
        // A name swapped for a junction or symlink between the two opens
        // leads the second to another file.
        if st.kind != seen || identity(&g).map_err(|e| read_failed(shown, e))? != first {
            return Err(changed(shown));
        }
        drop(f);
        Ok((g, st.size, st.mtime))
    }

    /// Lists a directory from its handle, with each entry's kind.
    fn list(d: &File, shown: &Path) -> Result<Vec<(OsString, Kind)>, String> {
        let mut out = Vec::new();
        // 8-byte aligned, as the records are.
        let mut buf = vec![0u64; 8192];
        loop {
            // SAFETY: `buf` is writable for the length passed.
            let ok = unsafe {
                GetFileInformationByHandleEx(
                    d.as_raw_handle() as HANDLE,
                    FileIdBothDirectoryInfo,
                    buf.as_mut_ptr().cast(),
                    (buf.len() * 8) as u32,
                )
            };
            if ok == 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                    return Ok(out);
                }
                return Err(read_failed(shown, e));
            }
            let base = buf.as_ptr().cast::<u8>();
            let end = buf.len() * 8;
            let mut at = 0usize;
            loop {
                let header = std::mem::offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);
                if at + header > end {
                    return Err(read_failed(shown, "a malformed directory listing"));
                }
                // SAFETY: the call wrote whole records into `buf`, each at
                // an 8-byte aligned offset the previous one's
                // `NextEntryOffset` gives, and the fixed part is in bounds.
                let rec = unsafe { base.add(at).cast::<FILE_ID_BOTH_DIR_INFO>() };
                // SAFETY: as above; fields are read through the raw pointer,
                // since the name runs past the struct's declared size.
                let (next, attributes, ea, name_bytes, name_at) = unsafe {
                    (
                        (*rec).NextEntryOffset,
                        (*rec).FileAttributes,
                        (*rec).EaSize,
                        (*rec).FileNameLength as usize,
                        std::ptr::addr_of!((*rec).FileName).cast::<u16>(),
                    )
                };
                if at + header + name_bytes > end {
                    return Err(read_failed(shown, "a malformed directory listing"));
                }
                // SAFETY: the name's `FileNameLength` bytes are in `buf`.
                let name = unsafe { std::slice::from_raw_parts(name_at, name_bytes / 2) };
                if name != [u16::from(b'.')] && name != [u16::from(b'.'); 2] {
                    // The reparse tag is in `EaSize` for a reparse point.
                    let kind = kind_of(attributes, ea);
                    out.push((OsString::from_wide(name), kind));
                }
                if next == 0 {
                    break;
                }
                at += next as usize;
            }
        }
    }

    /// Reads the target of the symlink or junction `name` in `parent`, and
    /// its mtime, from a handle on the link itself.
    fn read_link_at(parent: &File, name: &OsStr, shown: &Path) -> Result<(PathBuf, i64), String> {
        let f = open_at(parent, name, Open::Link, true).map_err(|e| read_failed(shown, e))?;
        let st = stat(&f).map_err(|e| read_failed(shown, e))?;
        if st.kind != Kind::Link {
            return Err(changed(shown));
        }
        // 4-byte aligned, as the buffer's fields are.
        let mut buf = vec![0u32; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize / 4];
        let mut got = 0u32;
        // SAFETY: `buf` is writable for the length passed, and the call is
        // synchronous.
        let ok = unsafe {
            DeviceIoControl(
                f.as_raw_handle() as HANDLE,
                FSCTL_GET_REPARSE_POINT,
                std::ptr::null(),
                0,
                buf.as_mut_ptr().cast(),
                MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
                &mut got,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(read_failed(shown, io::Error::last_os_error()));
        }
        // SAFETY: `buf` holds `got` initialised bytes and outlives `bytes`.
        let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), got as usize) };
        let target = super::reparse_target(bytes)
            .ok_or_else(|| read_failed(shown, "an unsupported or malformed reparse point"))?;
        Ok((PathBuf::from(target), st.mtime))
    }

    /// Appends one entry the walk saw in `parent`, and everything under it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn entry<W: Write>(
        b: &mut Builder<W>,
        parent: &File,
        name: &OsStr,
        seen: Kind,
        shown: &Path,
        entry: &Path,
        top: bool,
        progress: &Progress,
        walked: &mut Walked,
    ) -> Result<(), String> {
        match seen {
            Kind::Link if top => Err(changed(shown)),
            Kind::Link => {
                // Reads the link itself; nothing it points to is opened.
                let (target, mtime) = read_link_at(parent, name, shown)?;
                let target = PathBuf::from(target.to_string_lossy().replace('\\', "/"));
                progress.began(&entry.to_string_lossy());
                let mut h = header(EntryType::Symlink, 0o777, mtime, 0);
                b.append_link(&mut h, entry, &target)
                    .map_err(|e| read_failed(shown, e))
            }
            Kind::File => {
                let (f, size, mtime) = open_seen(parent, name, seen, shown)?;
                progress.began(&entry.to_string_lossy());
                let h = header(EntryType::Regular, 0o644, mtime, size);
                append_file(b, entry, h, f, size, shown)
            }
            Kind::Dir => {
                let (d, _, mtime) = open_seen(parent, name, seen, shown)?;
                if entry != Path::new(".") {
                    progress.began(&entry.to_string_lossy());
                    let mut h = header(EntryType::Directory, 0o755, mtime, 0);
                    b.append_data(&mut h, entry, io::empty())
                        .map_err(|e| read_failed(shown, e))?;
                }
                let mut children = list(&d, shown)?;
                children.sort_by(|a, b| a.0.cmp(&b.0));
                for (child, kind) in children {
                    let child_shown = shown.join(&child);
                    let child_entry = if entry == Path::new(".") {
                        PathBuf::from(&child)
                    } else {
                        entry.join(&child)
                    };
                    self::entry(
                        b,
                        &d,
                        &child,
                        kind,
                        &child_shown,
                        &child_entry,
                        false,
                        progress,
                        walked,
                    )?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(all(test, unix))]
pub(crate) fn test_entry<W: Write>(
    b: &mut Builder<W>,
    dirfd: &std::os::fd::OwnedFd,
    name: &str,
    seen: &rustix::fs::Stat,
    shown: &Path,
    walked: &mut Walked,
) -> Result<(), String> {
    let progress = Progress::default();
    let n = std::ffi::OsStr::new(name);
    unix::entry(
        b,
        dirfd,
        n,
        seen,
        shown,
        Path::new(name),
        false,
        &progress,
        walked,
    )
}

#[cfg(all(test, unix))]
pub(crate) fn test_append<W: Write>(
    b: &mut Builder<W>,
    file: std::fs::File,
    size: u64,
    shown: &Path,
) -> Result<(), String> {
    let h = header(EntryType::Regular, 0o644, 0, size);
    append_file(b, Path::new("data.bin"), h, file, size, shown)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `REPARSE_DATA_BUFFER` naming `subst` (and `print`).
    fn reparse(tag: u32, flags: Option<u32>, subst: &str, print: &str) -> Vec<u8> {
        let wide = |s: &str| -> Vec<u8> { s.encode_utf16().flat_map(u16::to_le_bytes).collect() };
        let (s, p) = (wide(subst), wide(print));
        let mut b = tag.to_le_bytes().to_vec();
        b.extend([0u8; 4]);
        for v in [0, s.len(), s.len(), p.len()] {
            b.extend((v as u16).to_le_bytes());
        }
        if let Some(f) = flags {
            b.extend(f.to_le_bytes());
        }
        b.extend(s);
        b.extend(p);
        b
    }

    #[test]
    fn a_reparse_point_s_target_reads_as_read_link_gives_it() {
        let sym = |flags, subst| {
            reparse_target(&reparse(
                IO_REPARSE_TAG_SYMLINK,
                Some(flags),
                subst,
                "shown",
            ))
        };
        assert_eq!(sym(1, r"..\lib").as_deref(), Some(r"..\lib"));
        assert_eq!(sym(0, r"\??\C:\data").as_deref(), Some(r"C:\data"));
        assert_eq!(
            sym(0, r"\??\UNC\host\share").as_deref(),
            Some(r"\\host\share")
        );
        assert_eq!(
            sym(0, r"\??\Volume{1}\x").as_deref(),
            Some(r"\\?\Volume{1}\x")
        );
        let junction = reparse(IO_REPARSE_TAG_MOUNT_POINT, None, r"\??\D:\src", "D:\\src");
        assert_eq!(reparse_target(&junction).as_deref(), Some(r"D:\src"));
        // Another tag, or offsets past the buffer, read as nothing.
        assert_eq!(
            reparse_target(&reparse(0x8000_0017, Some(0), "x", "x")),
            None
        );
        let mut cut = reparse(IO_REPARSE_TAG_SYMLINK, Some(1), "abc", "");
        cut.truncate(cut.len() - 2);
        assert_eq!(reparse_target(&cut), None);
    }

    #[test]
    fn a_poisoned_writer_sends_nothing() {
        let (tx, mut rx) = mpsc::channel(8);
        let mut w = ChunkWriter {
            tx,
            buf: Vec::new(),
            total: 0,
            progress: Arc::new(Progress::default()),
            poisoned: false,
        };
        w.write_all(&[1; 100]).unwrap();
        w.poisoned = true;
        w.write_all(&vec![0; 2 * CHUNK]).unwrap();
        w.flush().unwrap();
        drop(w);
        assert!(matches!(
            rx.try_recv(),
            Err(mpsc::error::TryRecvError::Disconnected)
        ));
    }
}
