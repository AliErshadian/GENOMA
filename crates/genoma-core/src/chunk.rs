use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const DEFAULT_CHUNK_SIZE: ChunkSize = ChunkSize::Mb1;

pub const SUPPORTED_CHUNK_SIZES: [ChunkSize; 7] = [
    ChunkSize::Kb4,
    ChunkSize::Kb16,
    ChunkSize::Kb64,
    ChunkSize::Mb1,
    ChunkSize::Mb4,
    ChunkSize::Mb16,
    ChunkSize::Mb64,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChunkSize {
    Kb4,
    Kb16,
    Kb64,
    Mb1,
    Mb4,
    Mb16,
    Mb64,
}

impl ChunkSize {
    pub fn bytes(self) -> usize {
        match self {
            Self::Kb4 => 4 * 1024,
            Self::Kb16 => 16 * 1024,
            Self::Kb64 => 64 * 1024,
            Self::Mb1 => 1024 * 1024,
            Self::Mb4 => 4 * 1024 * 1024,
            Self::Mb16 => 16 * 1024 * 1024,
            Self::Mb64 => 64 * 1024 * 1024,
        }
    }

    pub fn from_bytes(bytes: usize) -> Result<Self> {
        SUPPORTED_CHUNK_SIZES
            .iter()
            .copied()
            .find(|size| size.bytes() == bytes)
            .ok_or_else(|| Error::config(format!("unsupported chunk size: {bytes} bytes")))
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: u64,
    pub offset: u64,
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Streaming chunker. Never holds more than one chunk plus the reader buffer.
pub struct Chunker<R> {
    reader: R,
    chunk_size: usize,
    offset: u64,
    index: u64,
    done: bool,
}

impl<R: Read> Chunker<R> {
    pub fn new(reader: R, chunk_size: ChunkSize) -> Self {
        Self {
            reader,
            chunk_size: chunk_size.bytes(),
            offset: 0,
            index: 0,
            done: false,
        }
    }

    pub fn next_chunk(&mut self) -> Result<Option<Chunk>> {
        if self.done {
            return Ok(None);
        }

        let mut data = vec![0_u8; self.chunk_size];
        let mut filled = 0;
        while filled < self.chunk_size {
            let read = self.reader.read(&mut data[filled..])?;
            if read == 0 {
                break;
            }
            filled += read;
        }

        if filled == 0 {
            self.done = true;
            return Ok(None);
        }

        data.truncate(filled);
        let chunk = Chunk {
            index: self.index,
            offset: self.offset,
            data,
        };
        self.offset += filled as u64;
        self.index += 1;
        if filled < self.chunk_size {
            self.done = true;
        }
        Ok(Some(chunk))
    }
}

impl<R: Read> Iterator for Chunker<R> {
    type Item = Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_chunk() {
            Ok(Some(chunk)) => Some(Ok(chunk)),
            Ok(None) => None,
            Err(err) => {
                self.done = true;
                Some(Err(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn splits_even_stream() {
        let data = vec![7_u8; 8 * 1024];
        let chunks: Vec<Chunk> = Chunker::new(Cursor::new(data), ChunkSize::Kb4)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[1].offset, 4096);
        assert_eq!(chunks[1].index, 1);
    }

    #[test]
    fn last_chunk_may_be_short() {
        let data = vec![1_u8; 5000];
        let chunks: Vec<Chunk> = Chunker::new(Cursor::new(data), ChunkSize::Kb4)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[1].len(), 5000 - 4096);
    }

    #[test]
    fn ten_megabyte_stream_keeps_bounded_chunk_size() {
        const TOTAL: usize = 10 * 1024 * 1024;
        let data = vec![0xA5_u8; TOTAL];
        let mut max_held = 0;
        let mut counted = 0_u64;
        let mut bytes = 0_u64;
        for chunk in Chunker::new(Cursor::new(data), ChunkSize::Mb1) {
            let chunk = chunk.unwrap();
            max_held = max_held.max(chunk.len());
            counted += 1;
            bytes += chunk.len() as u64;
        }
        assert_eq!(bytes, TOTAL as u64);
        assert_eq!(counted, 10);
        assert!(max_held <= ChunkSize::Mb1.bytes());
    }
}
