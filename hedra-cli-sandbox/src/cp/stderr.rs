//! Reads a copy's stderr a line at a time for the copy's own bookkeeping.

use super::{LOSS_MARKER, NO_TAR_MARKER, TOTALS_ANNOUNCED};

/// The most stderr kept to show, and the longest line parsed.
const KEPT_MAX: usize = 64 << 10;

/// What the copy's stderr said: whether GNU tar ran, its totals, the
/// supervisor's loss marker, the no-`tar` marker, and the rest to show.
#[derive(Debug)]
pub(crate) struct CopyStderr {
    partial: Vec<u8>,
    pub(crate) kept: Vec<u8>,
    pub(crate) any: bool,
    pub(crate) gnu: bool,
    pub(crate) dropped: bool,
    pub(crate) no_tar: bool,
    pub(crate) totals: Option<u64>,
}

impl CopyStderr {
    pub(crate) fn new() -> Self {
        Self {
            partial: Vec::new(),
            kept: Vec::new(),
            any: false,
            gnu: false,
            dropped: false,
            no_tar: false,
            totals: None,
        }
    }

    pub(crate) fn write(&mut self, p: &[u8]) {
        self.any |= !p.is_empty();
        self.partial.extend_from_slice(p);
        while let Some(i) = self.partial.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.partial.drain(..=i).collect();
            self.line(&line);
        }
        if self.partial.len() > KEPT_MAX {
            self.flush();
        }
    }

    /// Takes a last line with no newline after it.
    pub(crate) fn flush(&mut self) {
        if !self.partial.is_empty() {
            let line = std::mem::take(&mut self.partial);
            self.line(&line);
        }
    }

    fn line(&mut self, l: &[u8]) {
        let text = String::from_utf8_lossy(l);
        if text.contains(TOTALS_ANNOUNCED) {
            self.gnu = true;
            return;
        }
        if text.contains(NO_TAR_MARKER) {
            self.no_tar = true;
            return;
        }
        if let Some(n) = totals(&text) {
            self.totals = Some(n);
            return;
        }
        if text.contains(LOSS_MARKER) {
            self.dropped = true;
        }
        let room = KEPT_MAX.saturating_sub(self.kept.len());
        self.kept.extend_from_slice(&l[..l.len().min(room)]);
    }
}

/// The count in GNU tar's `Total bytes written: N` line.
fn totals(line: &str) -> Option<u64> {
    let rest = &line[line.find("Total bytes written: ")? + "Total bytes written: ".len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bookkeeping_split_across_chunks() {
        let mut s = CopyStderr::new();
        s.write(format!("{TOTALS_ANNOUNCED}\ntar: a warning\nTotal bytes wr").as_bytes());
        s.write(b"itten: 20480 (20KiB, 1.0MiB/s)\n");
        s.flush();
        assert!(s.gnu);
        assert_eq!(s.totals, Some(20480));
        assert_eq!(s.kept, b"tar: a warning\n");
        assert!(!s.dropped);
    }
}
