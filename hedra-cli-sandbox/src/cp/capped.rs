//! A reader that follows a tar stream's headers as the `tar` crate reads
//! them, and fails before an extension entry over [`EXTENSION_MAX`] is read,
//! since the crate reads those into memory whole, and at a GNU sparse header,
//! whose chain of extension blocks the crate reads without limit.

use std::io::{self, Read};

use tar::{EntryType, Header, PaxExtensions};

use super::names::show;

/// The largest GNU long name or long link, or PAX extension entry, read; Go's
/// `archive/tar` holds them to the same.
pub(crate) const EXTENSION_MAX: u64 = 1 << 20;

const BLOCK: u64 = 512;

pub(crate) struct Capped<R> {
    inner: R,
    pos: u64,
    /// Where the next header starts.
    next: u64,
    header: Vec<u8>,
    /// A local PAX entry or GNU long name being read, and where it ends.
    ext: Option<(EntryType, Vec<u8>, u64)>,
    /// The last local PAX entry, which applies to the next entry returned.
    pending: Option<Vec<u8>>,
    /// The last GNU long name, which names the next entry returned.
    long_name: Option<Vec<u8>>,
    /// Past the end blocks, or past a header this reader cannot follow.
    done: bool,
    /// Past the end blocks.
    ended: bool,
}

impl<R: Read> Capped<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            pos: 0,
            next: 0,
            header: Vec::with_capacity(BLOCK as usize),
            ext: None,
            pending: None,
            long_name: None,
            done: false,
            ended: false,
        }
    }

    /// Whether the archive's end blocks have been read.
    pub(crate) fn ended(&self) -> bool {
        self.ended
    }

    fn take(&mut self, mut b: &[u8]) -> io::Result<()> {
        while !b.is_empty() && !self.done {
            if self.pos < self.next {
                let n = (self.next - self.pos).min(b.len() as u64) as usize;
                if let Some((kind, buf, end)) = &mut self.ext {
                    let keep = (*end - self.pos).min(n as u64) as usize;
                    buf.extend_from_slice(&b[..keep]);
                    if self.pos + keep as u64 == *end {
                        let kind = *kind;
                        let buf = self.ext.take().map(|(_, buf, _)| buf);
                        if kind == EntryType::XHeader {
                            self.pending = buf;
                        } else {
                            self.long_name = buf;
                        }
                    }
                }
                self.pos += n as u64;
                b = &b[n..];
                continue;
            }
            let n = (BLOCK as usize - self.header.len()).min(b.len());
            self.header.extend_from_slice(&b[..n]);
            self.pos += n as u64;
            b = &b[n..];
            if self.header.len() == BLOCK as usize {
                let header = std::mem::take(&mut self.header);
                self.header_read(&header)?;
            }
        }
        Ok(())
    }

    /// Follows the `tar` crate's `next_entry`: an extension entry describes
    /// the next one, and a local PAX `size` overrides that entry's size.
    fn header_read(&mut self, raw: &[u8]) -> io::Result<()> {
        if raw.iter().all(|b| *b == 0) {
            self.done = true;
            self.ended = true;
            return Ok(());
        }
        let h = Header::from_byte_slice(raw);
        let kind = h.entry_type();
        if kind.is_gnu_sparse() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("refused {:?}: it is a sparse file", show(&self.name_of(h))),
            ));
        }
        let extension = matches!(
            kind,
            EntryType::GNULongName
                | EntryType::GNULongLink
                | EntryType::XHeader
                | EntryType::XGlobalHeader
        );
        let Ok(mut size) = h.entry_size() else {
            // The `tar` crate fails on it too.
            self.done = true;
            return Ok(());
        };
        if extension && size > EXTENSION_MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "the archive has an extension entry of {size} bytes, over the {EXTENSION_MAX}-byte limit"
                ),
            ));
        }
        let recognized = h.as_gnu().is_some() || h.as_ustar().is_some();
        if !extension {
            if let Some(s) = self.pending.as_deref().and_then(pax_size) {
                size = s;
            }
        }
        let consumed = recognized
            && matches!(
                kind,
                EntryType::GNULongName | EntryType::GNULongLink | EntryType::XHeader
            );
        if !consumed {
            self.pending = None;
            self.long_name = None;
        }
        let Some(padded) = size.checked_add(BLOCK - 1).map(|s| s & !(BLOCK - 1)) else {
            self.done = true;
            return Ok(());
        };
        self.next = self.pos + padded;
        if recognized && matches!(kind, EntryType::XHeader | EntryType::GNULongName) {
            if size == 0 {
                if kind == EntryType::XHeader {
                    self.pending = Some(Vec::new());
                }
            } else {
                self.ext = Some((kind, Vec::with_capacity(size as usize), self.pos + size));
            }
        }
        Ok(())
    }

    /// The name the `tar` crate would give the entry `h` heads.
    fn name_of(&self, h: &Header) -> String {
        if let Some(n) = &self.long_name {
            let n = n.strip_suffix(&[0]).unwrap_or(n);
            return String::from_utf8_lossy(n).into_owned();
        }
        if let Some(p) = self.pending.as_deref().and_then(pax_path) {
            return p;
        }
        String::from_utf8_lossy(&h.path_bytes()).into_owned()
    }
}

/// The first PAX `path`, if every record before it parses.
fn pax_path(pax: &[u8]) -> Option<String> {
    for ext in PaxExtensions::new(pax) {
        let ext = ext.ok()?;
        if ext.key() == Ok("path") {
            return Some(String::from_utf8_lossy(ext.value_bytes()).into_owned());
        }
    }
    None
}

/// The `tar` crate's reading of a PAX `size`: the first `size` record, if
/// every record before it parses.
fn pax_size(pax: &[u8]) -> Option<u64> {
    for ext in PaxExtensions::new(pax) {
        let ext = ext.ok()?;
        if ext.key() != Ok("size") {
            continue;
        }
        return ext.value().ok()?.parse().ok();
    }
    None
}

impl<R: Read> Read for Capped<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.take(&buf[..n])?;
        Ok(n)
    }
}
