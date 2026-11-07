//! Reads a copy's stderr a line at a time for the copy's own bookkeeping.

use super::{COULD_NOT_START, LOSS_MARKER, NO_TAR_MARKER, TOTALS_ANNOUNCED};

/// The most stderr kept to show, and the longest line parsed.
const KEPT_MAX: usize = 64 << 10;

/// What the copy's stderr said: whether GNU tar ran, its totals, the
/// supervisor's loss marker and its line that the command could not start,
/// the no-`tar` marker, and the rest to show.
#[derive(Debug)]
pub(crate) struct CopyStderr {
    partial: Vec<u8>,
    pub(crate) kept: Vec<u8>,
    pub(crate) any: bool,
    pub(crate) gnu: bool,
    pub(crate) dropped: bool,
    pub(crate) no_tar: bool,
    /// The reason in the supervisor's line that the command could not start.
    pub(crate) could_not_start: Option<String>,
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
            could_not_start: None,
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
        if let Some(why) = text.strip_prefix(COULD_NOT_START) {
            self.could_not_start = Some(why.trim().to_string());
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

    #[test]
    fn reads_the_supervisors_line_that_the_command_could_not_start() {
        let mut s = CopyStderr::new();
        s.write(b"[hedra] the command could not start: exec: \"sh\": executable file not found in $PATH\r\n");
        s.flush();
        assert_eq!(
            s.could_not_start.as_deref(),
            Some("exec: \"sh\": executable file not found in $PATH")
        );
        assert!(s.kept.is_empty());
    }

    #[test]
    fn neither_the_kept_stderr_nor_a_partial_line_passes_its_cap() {
        let mut s = CopyStderr::new();
        let line = [b'x'; 62].iter().chain(b"\n").copied().collect::<Vec<u8>>();
        for _ in 0..4096 {
            s.write(&line);
        }
        assert_eq!(s.kept.len(), KEPT_MAX);
        let chunk = vec![b'y'; 4096];
        for _ in 0..256 {
            s.write(&chunk);
            assert!(s.partial.len() <= KEPT_MAX, "{}", s.partial.len());
        }
        s.flush();
        assert!(s.partial.is_empty());
        assert_eq!(s.kept.len(), KEPT_MAX);
    }
}
