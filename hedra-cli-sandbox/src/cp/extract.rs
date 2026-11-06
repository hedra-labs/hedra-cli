//! Copy out: writes a tar stream's entries under the receiving directory.
//!
//! Every write goes through a `cap_std::fs::Dir`, so no entry leaves the
//! copy through `..`, an absolute path or a symlink. The top entry is written
//! in the receiving directory, and everything beneath it through a `Dir`
//! opened at the top itself. `cap_std` keeps a path inside its `Dir` but
//! follows a symlink that stays inside, so each entry's parent is opened one
//! component at a time, refusing a symlink, and the entry is written through
//! that.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{BuildHasher, RandomState};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use cap_fs_ext::DirExt;
use cap_std::fs::{Dir, DirBuilder, OpenOptions};
use tar::EntryType;
use tokio::sync::mpsc;
use unicode_normalization::UnicodeNormalization;

use super::capped::Capped;
use super::names::{local_name, show};
use super::Progress;

/// The error of an extraction stopped by a cancel.
const CANCELLED: &str = "the copy was cancelled";

/// What the receiving task hands the extractor.
pub(crate) enum Down {
    Data(Bytes),
    /// The command exited: the stream is whole.
    End,
}

/// What an extraction did, or why it stopped.
#[derive(Default)]
pub(crate) struct Extracted {
    pub(crate) error: Option<String>,
    /// The stream ended before the archive did.
    pub(crate) cut: bool,
    /// The archive's end blocks arrived.
    pub(crate) whole: bool,
    pub(crate) skipped: Vec<String>,
    pub(crate) warnings: Vec<String>,
    /// Directories already there that were given owner rwx to take this
    /// copy's entries, with their modes, for a failed copy to put back.
    loosened: Vec<Loosened>,
}

/// A directory [`add_owner_rwx`] changed, by its root and name there, and
/// its mode before. By name, not held open, so a copy over many read-only
/// directories holds no descriptor for each. None on Windows, where modes
/// are not set.
struct Loosened {
    root: Root,
    rel: String,
    mode: u32,
}

/// Puts back the modes of the directories in `loosened`, beneath `dir` and
/// the top `name`, without following a symlink. Best effort, as loosening
/// them was.
fn restore_modes(dir: &Path, name: &str, loosened: &[Loosened]) {
    if loosened.is_empty() {
        return;
    }
    let Ok(outer) = Dir::open_ambient_dir(dir, cap_std::ambient_authority()) else {
        return;
    };
    let inner = (name != ".")
        .then(|| outer.open_dir_nofollow(name).ok())
        .flatten();
    for l in loosened.iter().rev() {
        let d = match l.root {
            Root::Outer => &outer,
            Root::Inner => match &inner {
                Some(d) => d,
                None => continue,
            },
        };
        if let Some(fd) = open_dir_fd(d, &l.rel) {
            let _ = set_mode(&fd, l.mode);
        }
    }
}

/// A `Read` over the chunks the receiving task hands over. A channel closed
/// without [`Down::End`] is an error, and sets `cut`; so is any read once
/// the copy was cancelled, so a file being written stops at once.
struct ChannelReader<'a> {
    rx: mpsc::Receiver<Down>,
    cur: Bytes,
    ended: bool,
    cut: Arc<AtomicBool>,
    progress: &'a Progress,
}

impl Read for ChannelReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.progress.cancelled() {
            return Err(io::Error::other(CANCELLED));
        }
        while self.cur.is_empty() {
            if self.ended {
                return Ok(0);
            }
            match self.rx.blocking_recv() {
                Some(Down::Data(b)) => self.cur = b,
                Some(Down::End) => self.ended = true,
                None => {
                    self.cut.store(true, Ordering::SeqCst);
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "the stream ended before the archive did",
                    ));
                }
            }
        }
        let n = buf.len().min(self.cur.len());
        buf[..n].copy_from_slice(&self.cur.split_to(n));
        Ok(n)
    }
}

/// Extracts the stream in `rx` under `dir`, renaming its top entry `base` to
/// `name`, then reads the rest of the stream so the sender never blocks.
pub(crate) fn extract(
    rx: mpsc::Receiver<Down>,
    dir: &Path,
    base: &str,
    name: &str,
    progress: &Progress,
) -> Extracted {
    extract_probing(rx, dir, base, name, progress, &probe)
}

/// [`extract`], learning what each directory folds from `probe`.
pub(crate) fn extract_probing(
    rx: mpsc::Receiver<Down>,
    dir: &Path,
    base: &str,
    name: &str,
    progress: &Progress,
    probe: &dyn Fn(&Dir) -> Folding,
) -> Extracted {
    let cut = Arc::new(AtomicBool::new(false));
    let mut r = ChannelReader {
        rx,
        cur: Bytes::new(),
        ended: false,
        cut: cut.clone(),
        progress,
    };
    let mut out = Extracted::default();
    let result = extract_from(&mut r, dir, base, name, progress, probe, &mut out);
    // A whole copy has set every directory's mode by now; a failed one
    // leaves the user's directories as they were.
    let loosened = std::mem::take(&mut out.loosened);
    if let Err(e) = result {
        restore_modes(dir, name, &loosened);
        out.error = Some(e);
        out.cut = cut.load(Ordering::SeqCst);
        return out;
    }
    // The record's padding after the archive's end blocks.
    let _ = io::copy(&mut r, &mut io::sink());
    out
}

/// Which `Dir` an entry is written through.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Root {
    Outer,
    Inner,
}

struct Extraction {
    outer: Dir,
    inner: Option<Dir>,
    name: String,
}

impl Extraction {
    /// The root an entry's local name is written through, and its name
    /// there. A top the stream has no entry for is made here.
    fn at(&mut self, local: &str, progress: &Progress) -> io::Result<(Root, String)> {
        if self.name == "." || local == self.name {
            return Ok((Root::Outer, local.to_string()));
        }
        if self.inner.is_none() {
            if !is_dir(&self.outer, &self.name) && !progress.writes() {
                return Err(io::Error::other(CANCELLED));
            }
            match self.outer.create_dir_with(&self.name, &private_dir()) {
                Ok(()) => progress.landed(),
                Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
                Err(_) => {}
            }
            self.inner = Some(self.outer.open_dir_nofollow(&self.name)?);
        }
        let rest = local
            .strip_prefix(&format!("{}/", self.name))
            .unwrap_or(local);
        Ok((Root::Inner, rest.to_string()))
    }

    fn dir(&self, root: Root) -> &Dir {
        match root {
            Root::Outer => &self.outer,
            Root::Inner => self.inner.as_ref().expect("opened by at"),
        }
    }
}

fn private_dir() -> DirBuilder {
    #[allow(unused_mut)]
    let mut b = DirBuilder::new();
    #[cfg(unix)]
    {
        use cap_std::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b
}

struct Pending {
    root: Root,
    /// Components in the entry's local name.
    depth: usize,
    local: String,
    rel: String,
    mode: u32,
    mtime: SystemTime,
}

struct Link {
    root: Root,
    rel: String,
    local: String,
    target: PathBuf,
}

/// A symlink's target as the archive holds it: on Unix byte for byte, as
/// tar writes it, whether or not it is UTF-8.
fn link_target(raw: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(std::ffi::OsStr::from_bytes(raw))
    }
    #[cfg(not(unix))]
    PathBuf::from(String::from_utf8_lossy(raw).into_owned())
}

fn mtime_of(h: &tar::Header) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(h.mtime().unwrap_or(0))
}

fn extract_from(
    r: &mut ChannelReader<'_>,
    dir: &Path,
    base: &str,
    name: &str,
    progress: &Progress,
    probe: &dyn Fn(&Dir) -> Folding,
    out: &mut Extracted,
) -> Result<(), String> {
    let mask = umask();
    let outer = Dir::open_ambient_dir(dir, cap_std::ambient_authority())
        .map_err(|e| format!("cannot open {}: {e}", dir.display()))?;
    let outer_folding = probe(&outer);
    let mut collisions = Collisions::new(outer_folding);
    // Entries beneath the top are written in it, which, when it already
    // exists, may be on another file system.
    let mut inner_probed = name == ".";
    let mut x = Extraction {
        outer,
        inner: None,
        name: name.to_string(),
    };
    let mut dirs: Vec<Pending> = Vec::new();
    let mut links: Vec<Link> = Vec::new();
    let mut written = Written::new();

    let mut archive = tar::Archive::new(Capped::new(r));
    let entries = archive.entries().map_err(|e| show(&e.to_string()))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| show(&e.to_string()))?;
        let kind = entry.header().entry_type();
        if kind == EntryType::XGlobalHeader {
            continue;
        }
        let raw = entry.path_bytes().into_owned();
        let Ok(entry_name) = String::from_utf8(raw.clone()) else {
            return Err(format!(
                "refused \"{}\": its name is not UTF-8",
                show(&String::from_utf8_lossy(&raw))
            ));
        };
        let shown = show(&entry_name);
        let local = local_name(&entry_name, base, name)?;
        if local == name && !matches!(kind, EntryType::Directory | EntryType::Regular) {
            return Err(format!(
                "refused \"{shown}\": the copy's top is not a file or a directory"
            ));
        }
        // `Capped` refuses a GNU sparse header before the crate reads it.
        if is_pax_sparse(&mut entry) {
            return Err(format!("refused \"{shown}\": it is a sparse file"));
        }
        let local_shown = show(&local);
        #[cfg(windows)]
        {
            if let Some(why) = local
                .split('/')
                .filter(|c| *c != ".")
                .find_map(|c| super::names::windows_name_problem(c))
            {
                out.warnings.push(format!("left out {local_shown}: {why}"));
                out.skipped.push(local_shown);
                continue;
            }
        }
        if !inner_probed && local != name {
            inner_probed = true;
            if let Ok(inner) = x.outer.open_dir_nofollow(name) {
                collisions.folding = probe(&inner);
            }
        }
        if let Some(prev) = collisions.check(&local) {
            out.warnings.push(format!(
                "left out {local_shown}: the destination does not tell its name from {}",
                show(&prev)
            ));
            out.skipped.push(local_shown);
            continue;
        }
        if !progress.began(&local) {
            return Err(CANCELLED.into());
        }
        let (root, rel) = x
            .at(&local, progress)
            .map_err(|e| format!("{local_shown}: {e}"))?;
        let mode = entry.header().mode().unwrap_or(0o644) & 0o777 & !mask;
        let mtime = mtime_of(entry.header());
        match kind {
            EntryType::Directory => {
                // A directory already there is no write of this copy's.
                if !is_dir(x.dir(root), &rel) {
                    will_write(progress)?;
                }
                if mkdir_over(x.dir(root), &rel).map_err(|e| failed(&local, &rel, e))? {
                    progress.landed();
                }
                if rel != "." {
                    // A directory an earlier copy left read-only takes this
                    // copy's entries; its mode is set again once they are,
                    // or put back when the copy fails.
                    if let Some(mode) = add_owner_rwx(x.dir(root), &rel) {
                        out.loosened.push(Loosened {
                            root,
                            rel: rel.clone(),
                            mode,
                        });
                    }
                    dirs.push(Pending {
                        root,
                        depth: local.split('/').count(),
                        local: local.clone(),
                        rel,
                        mode,
                        mtime,
                    });
                }
            }
            EntryType::Regular => {
                will_write(progress)?;
                write_file(x.dir(root), &rel, mode, mtime, &mut entry, progress)
                    .map_err(|e| failed(&local, &rel, e))?;
                written.insert(&local);
            }
            EntryType::Symlink => {
                let target = link_target(&entry.link_name_bytes().unwrap_or_default());
                links.push(Link {
                    root,
                    rel,
                    local,
                    target,
                });
            }
            EntryType::Link => {
                let raw = entry.link_name_bytes().unwrap_or_default().into_owned();
                let Ok(target_raw) = String::from_utf8(raw) else {
                    return Err(format!(
                        "refused \"{shown}\": a hard link whose target is not UTF-8"
                    ));
                };
                let target = local_name(&target_raw, base, name)?;
                // Only a file this copy wrote, so a merge into an existing
                // directory cannot link to a file the user already had.
                if !written.contains(&target) {
                    return Err(format!(
                        "refused \"{shown}\": a hard link to \"{}\", which this copy did not write",
                        show(&target_raw)
                    ));
                }
                will_write(progress)?;
                let (troot, trel) = x
                    .at(&target, progress)
                    .map_err(|e| format!("{local_shown}: {e}"))?;
                let (from, from_leaf) =
                    parent(x.dir(troot), &trel).map_err(|e| failed(&target, &trel, e))?;
                let (to, leaf) = parent(x.dir(root), &rel).map_err(|e| failed(&local, &rel, e))?;
                replace(&to, leaf).map_err(|e| failed(&local, &rel, e))?;
                from.hard_link(from_leaf, &to, leaf)
                    .map_err(|e| failed(&local, &rel, e))?;
                progress.landed();
                written.insert(&local);
            }
            _ => {
                out.warnings.push(format!(
                    "left out {local_shown}: not a file, a directory or a link"
                ));
                out.skipped.push(local_shown);
            }
        }
    }
    // A stream cut at a header's boundary reads as a clean end to the `tar`
    // crate; only the end blocks say the archive is whole.
    out.whole = archive.into_inner().ended();

    let made: HashSet<&str> = links.iter().map(|l| l.local.as_str()).collect();
    for l in &links {
        let local_shown = show(&l.local);
        if cfg!(windows) {
            out.warnings.push(format!(
                "left out {local_shown}: symlinks are not created on Windows"
            ));
            out.skipped.push(local_shown);
            continue;
        }
        if let Some(via) = through_link(&l.local, &made) {
            return Err(format!(
                "refused \"{local_shown}\": it is beneath \"{}\", a symlink in the same stream",
                show(via)
            ));
        }
        will_write(progress)?;
        let (d, leaf) = parent(x.dir(l.root), &l.rel).map_err(|e| failed(&l.local, &l.rel, e))?;
        replace(&d, leaf).map_err(|e| failed(&l.local, &l.rel, e))?;
        symlink(&d, &l.target, leaf).map_err(|e| failed(&l.local, &l.rel, e))?;
        progress.landed();
    }

    // Deepest first, so a read-only directory gets its mode only once
    // everything under it is written, and no later write changes its mtime;
    // a path's later entry wins.
    dirs.sort_by_key(|d| std::cmp::Reverse(d.depth));
    for d in &dirs {
        set_dir_attrs(x.dir(d.root), &d.rel, d.mode, d.mtime)
            .map_err(|e| failed(&d.local, &d.rel, e))?;
    }
    Ok(())
}

/// Records that the copy is about to change the destination, so a cancelled
/// copy knows whether it wrote anything; an error once it was cancelled.
fn will_write(progress: &Progress) -> Result<(), String> {
    if progress.writes() {
        Ok(())
    } else {
        Err(CANCELLED.into())
    }
}

/// Whether `rel` under `d` is a directory already, read without following a
/// symlink.
fn is_dir(d: &Dir, rel: &str) -> bool {
    parent(d, rel)
        .and_then(|(p, leaf)| p.symlink_metadata(leaf))
        .is_ok_and(|m| m.is_dir())
}

fn is_pax_sparse<R: Read>(entry: &mut tar::Entry<'_, R>) -> bool {
    let Ok(Some(exts)) = entry.pax_extensions() else {
        return false;
    };
    exts.flatten()
        .any(|e| e.key_bytes().starts_with(b"GNU.sparse"))
}

/// The proper ancestor of `local` that is in `links`, if any.
fn through_link<'a>(local: &str, links: &HashSet<&'a str>) -> Option<&'a str> {
    local
        .match_indices('/')
        .find_map(|(i, _)| links.get(&local[..i]).copied())
}

/// Which names a destination tells apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Folding {
    pub(crate) case: bool,
    pub(crate) normalization: bool,
}

/// The files a copy out wrote, by local name, for a hard link to name one.
/// Each name is held as two 64-bit hashes under separate secret keys, so a
/// copy of many small files holds no names: a name the copy did not write
/// reads as written only when both of its hashes match one name it did.
pub(crate) struct Written<A = RandomState, B = RandomState> {
    first: A,
    second: B,
    names: HashSet<(u64, u64)>,
}

impl Written {
    pub(crate) fn new() -> Self {
        Self::with_hashers(RandomState::new(), RandomState::new())
    }
}

impl<A: BuildHasher, B: BuildHasher> Written<A, B> {
    pub(crate) fn with_hashers(first: A, second: B) -> Self {
        Self {
            first,
            second,
            names: HashSet::new(),
        }
    }

    fn key(&self, name: &str) -> (u64, u64) {
        (self.first.hash_one(name), self.second.hash_one(name))
    }

    pub(crate) fn insert(&mut self, name: &str) {
        let key = self.key(name);
        let _ = self.names.insert(key);
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.names.contains(&self.key(name))
    }
}

/// Entry names that land on the same file in the destination: each folded
/// name is held as a keyed hash, beside the first name that folded to it,
/// which a collision's message names. A folded name whose hash an earlier,
/// different folded name has is held in full, so a hash collision never
/// reads as two names landing on the same file.
pub(crate) struct Collisions<S = RandomState> {
    pub(crate) folding: Folding,
    hasher: S,
    seen: HashMap<u64, Box<str>>,
    clashes: HashMap<String, Box<str>>,
}

impl Collisions {
    pub(crate) fn new(folding: Folding) -> Self {
        Self::with_hasher(folding, RandomState::new())
    }
}

impl<S: BuildHasher> Collisions<S> {
    pub(crate) fn with_hasher(folding: Folding, hasher: S) -> Self {
        Self {
            folding,
            hasher,
            seen: HashMap::new(),
            clashes: HashMap::new(),
        }
    }

    /// An earlier, different name that `local` lands on, if any.
    pub(crate) fn check(&mut self, local: &str) -> Option<String> {
        if !self.folding.case && !self.folding.normalization {
            return None;
        }
        let key = fold(self.folding, local);
        let hash = self.hasher.hash_one(&key);
        let first: &str = match self.seen.get(&hash) {
            None => {
                let _ = self.seen.insert(hash, local.into());
                return None;
            }
            Some(first) if fold(self.folding, first) == key => first,
            Some(_) => self.clashes.entry(key).or_insert_with(|| local.into()),
        };
        (first != local).then(|| first.to_string())
    }
}

/// `name` as a destination that folds as `folding` says sees it.
fn fold(folding: Folding, name: &str) -> String {
    let mut key = name.to_string();
    if folding.case {
        key = key.to_lowercase();
    }
    if folding.normalization {
        key = key.nfc().collect();
    }
    key
}

/// What the receiving directory's file system folds, probed by creating a
/// file there and looking it up by another case and another Unicode
/// normalization. A probe that cannot run assumes folding.
pub(crate) fn probe(d: &Dir) -> Folding {
    let id = std::process::id();
    let found = |made: &str, looked: &str| -> bool {
        // Never opens a file or link already there.
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        if d.open_with(made, &opts).is_err() {
            return true;
        }
        let found = d.symlink_metadata(looked).is_ok();
        let _ = d.remove_file(made);
        found
    };
    let probe = format!(".hedra-cp-probe-{id}");
    // "é" composed, then decomposed.
    let nfc = format!(".hedra-cp-probe-\u{e9}-{id}");
    let nfd = format!(".hedra-cp-probe-e\u{301}-{id}");
    Folding {
        case: found(&probe, &probe.to_uppercase()),
        normalization: found(&nfc, &nfd),
    }
}

/// A component of an entry's parent that is a symlink in the destination:
/// the part of the entry's path, under its root, that ends at the symlink.
#[derive(Debug)]
struct BeneathLink(String);

impl fmt::Display for BeneathLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" is a symlink", show(&self.0))
    }
}

impl std::error::Error for BeneathLink {}

/// The message for an entry `local`, written at `rel` under its root, that
/// failed with `e`.
fn failed(local: &str, rel: &str, e: io::Error) -> String {
    let local_shown = show(local);
    let Some(link) = e.get_ref().and_then(|i| i.downcast_ref::<BeneathLink>()) else {
        return format!("{local_shown}: {e}");
    };
    // `local` ends with `rel`, so the symlink's own local name is `local`
    // up to the end of `link.0`.
    let at = local.len() - rel.len() + link.0.len();
    format!(
        "refused \"{local_shown}\": it is beneath \"{}\", a symlink in the destination",
        show(local.get(..at).unwrap_or(&link.0))
    )
}

/// The directory holding `rel`'s last component, opened from `d` one
/// component at a time without following a symlink, and that component.
/// A symlink among the parent's components is a [`BeneathLink`] error.
fn parent<'a>(d: &Dir, rel: &'a str) -> io::Result<(Dir, &'a str)> {
    let Some((dirs, leaf)) = rel.rsplit_once('/') else {
        return Ok((d.try_clone()?, rel));
    };
    let mut at = d.try_clone()?;
    let mut end = 0;
    for c in dirs.split('/') {
        end += c.len();
        at = match at.open_dir_nofollow(c) {
            Ok(next) => next,
            Err(_) if at.symlink_metadata(c).is_ok_and(|m| m.is_symlink()) => {
                return Err(io::Error::other(BeneathLink(rel[..end].to_string())));
            }
            Err(e) => return Err(e),
        };
        end += 1;
    }
    Ok((at, leaf))
}

/// Makes a directory at `rel`, or keeps the one there, and says whether it
/// made one; anything else there is an error, since the mode and mtime set
/// later would follow a symlink.
pub(crate) fn mkdir_over(d: &Dir, rel: &str) -> io::Result<bool> {
    let (d, rel) = parent(d, rel)?;
    match d.create_dir_with(rel, &private_dir()) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            if d.symlink_metadata(rel)?.is_dir() {
                Ok(false)
            } else {
                Err(io::Error::other("it exists and is not a directory"))
            }
        }
        Err(e) => Err(e),
    }
}

/// Removes a file or symlink at `leaf`, a name in `d`, before a file or link
/// is written there, so the write never goes through an old symlink.
fn replace(d: &Dir, leaf: &str) -> io::Result<()> {
    match d.symlink_metadata(leaf) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(m) if m.is_dir() => Err(io::Error::other("a directory is in the way")),
        Ok(_) => d.remove_file(leaf),
    }
}

fn write_file(
    d: &Dir,
    rel: &str,
    mode: u32,
    mtime: SystemTime,
    body: &mut impl Read,
    progress: &Progress,
) -> io::Result<()> {
    let (d, rel) = parent(d, rel)?;
    replace(&d, rel)?;
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        opts.mode(mode);
    }
    #[cfg(not(unix))]
    let _ = mode;
    let f = d.open_with(rel, &opts)?;
    progress.landed();
    let mut f = f.into_std();
    io::copy(body, &mut f)?;
    f.set_modified(mtime)
}

#[cfg(unix)]
fn symlink(d: &Dir, target: &Path, rel: &str) -> io::Result<()> {
    d.symlink_contents(target, rel)
}

#[cfg(not(unix))]
fn symlink(_: &Dir, _: &Path, _: &str) -> io::Result<()> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// Gives the directory at `rel` its owner's read, write and search bits
/// when it lacks one, so entries can be written into a directory an earlier
/// copy left read-only, and returns its mode before when it changed it.
/// Best effort: a directory that cannot be opened for reading keeps its
/// mode, and writing beneath it fails naming the entry.
#[cfg(unix)]
fn add_owner_rwx(d: &Dir, rel: &str) -> Option<u32> {
    let fd = open_dir_fd(d, rel)?;
    let st = rustix::fs::fstat(&fd).ok()?;
    #[allow(clippy::useless_conversion)]
    let mode = u32::from(st.st_mode) & 0o7777;
    if mode & 0o700 == 0o700 {
        return None;
    }
    set_mode(&fd, mode | 0o700).then_some(mode)
}

#[cfg(not(unix))]
fn add_owner_rwx(_: &Dir, _: &str) -> Option<u32> {
    None
}

/// The directory at `rel` beneath `d`, opened for reading without following
/// a symlink.
#[cfg(unix)]
fn open_dir_fd(d: &Dir, rel: &str) -> Option<rustix::fd::OwnedFd> {
    use rustix::fs::{openat, Mode, OFlags};
    let (d, leaf) = parent(d, rel).ok()?;
    let dir = d.open_dir_nofollow(leaf).ok()?;
    // As in `set_dir_attrs`: cap-std's directory may be `O_PATH`.
    openat(
        &dir,
        ".",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .ok()
}

#[cfg(not(unix))]
fn open_dir_fd(_: &Dir, _: &str) -> Option<()> {
    None
}

/// Sets the mode of the directory open at `fd`; whether it did.
#[cfg(unix)]
fn set_mode(fd: &rustix::fd::OwnedFd, mode: u32) -> bool {
    rustix::fs::fchmod(fd, rustix::fs::Mode::from_raw_mode(mode as _)).is_ok()
}

#[cfg(not(unix))]
fn set_mode(_: &(), _: u32) -> bool {
    false
}

/// Sets a directory's mode and mtime, refusing a symlink at `rel`.
pub(crate) fn set_dir_attrs(d: &Dir, rel: &str, mode: u32, mtime: SystemTime) -> io::Result<()> {
    let (d, rel) = parent(d, rel)?;
    let dir = d.open_dir_nofollow(rel)?;
    #[cfg(unix)]
    {
        use rustix::fs::{openat, Mode, OFlags};
        use std::os::unix::fs::PermissionsExt;
        // cap-std opens a directory `O_PATH` on Linux, and `fchmod` and
        // `futimens` refuse an `O_PATH` descriptor with EBADF, so open the
        // same directory through it again for reading.
        let fd = openat(
            &dir,
            ".",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let f = std::fs::File::from(fd);
        f.set_permissions(std::fs::Permissions::from_mode(mode))?;
        f.set_modified(mtime)
    }
    #[cfg(not(unix))]
    {
        let f = dir.into_std_file();
        let _ = mode;
        let _ = f.set_modified(mtime);
        Ok(())
    }
}

/// The process's file mode creation mask.
#[cfg(unix)]
pub(crate) fn umask() -> u32 {
    static MASK: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *MASK.get_or_init(|| {
        #[cfg(target_os = "linux")]
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            if let Some(m) = status
                .lines()
                .find_map(|l| l.strip_prefix("Umask:"))
                .and_then(|v| u32::from_str_radix(v.trim(), 8).ok())
            {
                return m;
            }
        }
        use rustix::fs::Mode;
        let old = rustix::process::umask(Mode::from_raw_mode(0o022));
        rustix::process::umask(old);
        old.bits() as u32
    })
}

#[cfg(not(unix))]
pub(crate) fn umask() -> u32 {
    0
}
