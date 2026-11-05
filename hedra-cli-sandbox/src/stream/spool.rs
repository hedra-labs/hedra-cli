//! A FIFO of byte chunks that keeps the first `memory` bytes in memory and
//! the rest in an already-unlinked temporary file, so no exit leaves the
//! data on disk.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Spool {
    memory_limit: usize,
    total_limit: u64,
    memory: VecDeque<Vec<u8>>,
    in_memory: usize,
    file: Option<File>,
    /// File offsets written to and read from, length prefixes included.
    written: u64,
    read: u64,
    /// Payload bytes held in `file`.
    in_file: u64,
}

impl Spool {
    pub fn new(memory_limit: usize, total_limit: u64) -> Self {
        Self {
            memory_limit,
            total_limit,
            memory: VecDeque::new(),
            in_memory: 0,
            file: None,
            written: 0,
            read: 0,
            in_file: 0,
        }
    }

    /// Bytes held.
    pub fn len(&self) -> u64 {
        self.in_memory as u64 + self.in_file
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether another chunk fits under the total limit.
    pub fn has_room(&self) -> bool {
        self.len() < self.total_limit
    }

    /// Appends a chunk, spilling to the file once memory is full. Order is
    /// kept: once anything is in the file, every later chunk goes there.
    pub fn push(&mut self, chunk: &[u8]) -> io::Result<()> {
        let spilled = self.written > self.read;
        if !spilled && self.in_memory + chunk.len() <= self.memory_limit {
            self.in_memory += chunk.len();
            self.memory.push_back(chunk.to_vec());
            return Ok(());
        }
        let file = match &mut self.file {
            Some(f) => f,
            None => self.file.insert(tempfile::tempfile()?),
        };
        file.seek(SeekFrom::Start(self.written))?;
        file.write_all(&[&(chunk.len() as u32).to_le_bytes()[..], chunk].concat())?;
        self.written += 4 + chunk.len() as u64;
        self.in_file += chunk.len() as u64;
        Ok(())
    }

    /// Takes the oldest chunk.
    pub fn pop(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(chunk) = self.memory.pop_front() {
            self.in_memory -= chunk.len();
            return Ok(Some(chunk));
        }
        let Some(file) = &mut self.file else {
            return Ok(None);
        };
        if self.read == self.written {
            // Drained: start the file over.
            self.read = 0;
            self.written = 0;
            file.set_len(0)?;
            return Ok(None);
        }
        file.seek(SeekFrom::Start(self.read))?;
        let mut len = [0u8; 4];
        file.read_exact(&mut len)?;
        let mut chunk = vec![0u8; u32::from_le_bytes(len) as usize];
        file.read_exact(&mut chunk)?;
        self.read += 4 + chunk.len() as u64;
        self.in_file -= chunk.len() as u64;
        Ok(Some(chunk))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_come_back_in_order_across_memory_and_file() {
        let mut s = Spool::new(10, 1_000);
        for i in 0..20u8 {
            s.push(&[i; 3]).unwrap();
        }
        assert!(s.file.is_some());
        for i in 0..20u8 {
            assert_eq!(s.pop().unwrap().unwrap(), vec![i; 3]);
        }
        assert!(s.pop().unwrap().is_none());
        assert!(s.is_empty());
    }

    #[test]
    fn a_chunk_pushed_after_spilling_waits_behind_the_file() {
        let mut s = Spool::new(4, 1_000);
        s.push(b"ab").unwrap();
        s.push(b"cdef").unwrap(); // spills
        assert_eq!(s.pop().unwrap().unwrap(), b"ab");
        s.push(b"g").unwrap(); // memory has room, but must follow the file
        assert_eq!(s.pop().unwrap().unwrap(), b"cdef");
        assert_eq!(s.pop().unwrap().unwrap(), b"g");
    }

    #[test]
    fn room_is_bounded_by_the_total() {
        let mut s = Spool::new(2, 8);
        s.push(b"abcd").unwrap();
        assert!(s.has_room());
        s.push(b"efgh").unwrap();
        assert!(!s.has_room());
        s.pop().unwrap();
        assert!(s.has_room());
    }
}
