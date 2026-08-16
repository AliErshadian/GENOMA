//! Deterministic π digit access.
//!
//! Digits are loaded from a local dataset. GENOMA never downloads π at runtime.
//! Offsets past the end of the dataset wrap, and wrap metadata is recorded.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use genoma_core::{Error, Result};

pub const EXPECTED_PREFIX: &[u8] = b"14159265358979323846";

pub trait PiSource {
    fn len(&self) -> u64;
    fn get_digits(&self, offset: u64, length: usize) -> Result<Vec<u8>>;

    fn get_digits_with_wrap(&self, offset: u64, length: usize) -> Result<PiSlice> {
        let digits = self.get_digits(offset, length)?;
        let len = self.len();
        let wrapped = len > 0 && (offset >= len || offset.saturating_add(length as u64) > len);
        let wrap_count = if len == 0 {
            0
        } else {
            (offset / len) + u64::from(offset % len + length as u64 > len)
        };
        Ok(PiSlice {
            digits,
            offset,
            wrapped,
            wrap_count,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PiSlice {
    pub digits: Vec<u8>,
    pub offset: u64,
    pub wrapped: bool,
    pub wrap_count: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryPiSource {
    digits: Vec<u8>,
}

impl MemoryPiSource {
    pub fn new(digits: Vec<u8>) -> Result<Self> {
        validate_digits(&digits)?;
        Ok(Self { digits })
    }

    pub fn from_ascii(ascii: &[u8]) -> Result<Self> {
        Self::new(ascii.to_vec())
    }
}

impl PiSource for MemoryPiSource {
    fn len(&self) -> u64 {
        self.digits.len() as u64
    }

    fn get_digits(&self, offset: u64, length: usize) -> Result<Vec<u8>> {
        read_wrapping(&self.digits, offset, length)
    }
}

#[derive(Debug, Clone)]
pub struct FilePiSource {
    path: PathBuf,
    digits: Vec<u8>,
}

impl FilePiSource {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let digits = fs::read(&path)?;
        validate_digits(&digits)?;
        Ok(Self { path, digits })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl PiSource for FilePiSource {
    fn len(&self) -> u64 {
        self.digits.len() as u64
    }

    fn get_digits(&self, offset: u64, length: usize) -> Result<Vec<u8>> {
        read_wrapping(&self.digits, offset, length)
    }
}

pub struct CachedPiSource<S: PiSource> {
    inner: S,
    cache: Mutex<Vec<CacheEntry>>,
    capacity: usize,
}

#[derive(Clone)]
struct CacheEntry {
    offset: u64,
    length: usize,
    digits: Vec<u8>,
}

impl<S: PiSource> CachedPiSource<S> {
    pub fn new(inner: S, capacity: usize) -> Self {
        Self {
            inner,
            cache: Mutex::new(Vec::new()),
            capacity: capacity.max(1),
        }
    }
}

impl<S: PiSource> PiSource for CachedPiSource<S> {
    fn len(&self) -> u64 {
        self.inner.len()
    }

    fn get_digits(&self, offset: u64, length: usize) -> Result<Vec<u8>> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache
                .iter()
                .find(|entry| entry.offset == offset && entry.length == length)
            {
                return Ok(entry.digits.clone());
            }
        }
        let digits = self.inner.get_digits(offset, length)?;
        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() >= self.capacity {
                cache.remove(0);
            }
            cache.push(CacheEntry {
                offset,
                length,
                digits: digits.clone(),
            });
        }
        Ok(digits)
    }
}

fn validate_digits(digits: &[u8]) -> Result<()> {
    if digits.is_empty() {
        return Err(Error::pi("π dataset is empty"));
    }
    if !digits.iter().all(|d| (*d).is_ascii_digit()) {
        return Err(Error::pi("π dataset must contain ASCII digits 0-9"));
    }
    if digits.len() >= EXPECTED_PREFIX.len() && !digits.starts_with(EXPECTED_PREFIX) {
        return Err(Error::pi(
            "π dataset prefix does not match the known decimal expansion",
        ));
    }
    Ok(())
}

fn read_wrapping(digits: &[u8], offset: u64, length: usize) -> Result<Vec<u8>> {
    if digits.is_empty() {
        return Err(Error::pi("π dataset is empty"));
    }
    let len = digits.len() as u64;
    let mut out = Vec::with_capacity(length);
    let mut pos = offset % len;
    for _ in 0..length {
        out.push(digits[pos as usize]);
        pos += 1;
        if pos == len {
            pos = 0;
        }
    }
    Ok(out)
}

pub fn digits_to_unit(digits: &[u8]) -> f64 {
    if digits.is_empty() {
        return 0.0;
    }
    let take = digits.len().min(12);
    let mut acc = 0.0;
    let mut place = 0.1;
    for &digit in &digits[..take] {
        acc += f64::from(digit.saturating_sub(b'0')) * place;
        place *= 0.1;
    }
    acc
}

pub fn four_digit_group(digits: &[u8]) -> u32 {
    let mut value = 0_u32;
    for &digit in digits.iter().take(4) {
        value = value * 10 + u32::from(digit.saturating_sub(b'0'));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"1415926535897932384626433832795028841971";

    #[test]
    fn prefix_and_range() {
        let source = MemoryPiSource::from_ascii(SAMPLE).unwrap();
        assert_eq!(&source.get_digits(0, 10).unwrap(), b"1415926535");
        assert_eq!(&source.get_digits(5, 5).unwrap(), b"26535");
    }

    #[test]
    fn wrap_metadata() {
        let source = MemoryPiSource::from_ascii(SAMPLE).unwrap();
        let slice = source
            .get_digits_with_wrap(SAMPLE.len() as u64 + 2, 4)
            .unwrap();
        assert!(slice.wrapped);
        assert!(slice.wrap_count >= 1);
        assert_eq!(slice.digits, b"1592");
    }

    #[test]
    fn cache_hits() {
        let source = CachedPiSource::new(MemoryPiSource::from_ascii(SAMPLE).unwrap(), 4);
        let a = source.get_digits(3, 8).unwrap();
        let b = source.get_digits(3, 8).unwrap();
        assert_eq!(a, b);
    }
}
