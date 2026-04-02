//! Deterministic linear memory implementation for AOXCVM phase-1 execution.

/// Errors produced by [`LinearMemory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// The requested memory range exceeds configured bounds.
    OutOfBounds,
}

/// Bounded linear memory with deterministic zero-initialization behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearMemory {
    bytes: Vec<u8>,
    max_len: usize,
}

impl LinearMemory {
    /// Creates memory with an initial size and a hard maximum size.
    pub fn new(initial_len: usize, max_len: usize) -> Self {
        let init = initial_len.min(max_len);
        Self {
            bytes: vec![0; init],
            max_len,
        }
    }

    /// Current memory length in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if memory has zero length.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Ensures `[offset, offset + len)` is available and zero-fills grown region.
    pub fn ensure(&mut self, offset: usize, len: usize) -> Result<(), MemoryError> {
        let end = offset.checked_add(len).ok_or(MemoryError::OutOfBounds)?;
        if end > self.max_len {
            return Err(MemoryError::OutOfBounds);
        }
        if end > self.bytes.len() {
            self.bytes.resize(end, 0);
        }
        Ok(())
    }

    /// Reads a little-endian `u64` from memory.
    pub fn read_u64(&self, offset: usize) -> Result<u64, MemoryError> {
        let end = offset.checked_add(8).ok_or(MemoryError::OutOfBounds)?;
        let chunk = self
            .bytes
            .get(offset..end)
            .ok_or(MemoryError::OutOfBounds)?;
        let mut tmp = [0_u8; 8];
        tmp.copy_from_slice(chunk);
        Ok(u64::from_le_bytes(tmp))
    }

    /// Writes a little-endian `u64` into memory.
    pub fn write_u64(&mut self, offset: usize, value: u64) -> Result<(), MemoryError> {
        self.ensure(offset, 8)?;
        self.bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LinearMemory, MemoryError};

    #[test]
    fn writes_expand_with_zero_fill() {
        let mut memory = LinearMemory::new(0, 32);
        memory.write_u64(8, 42).expect("should fit");
        assert_eq!(memory.len(), 16);
        assert_eq!(memory.read_u64(8).expect("read back"), 42);
        assert_eq!(memory.read_u64(0).expect("zero-filled"), 0);
    }

    #[test]
    fn out_of_bounds_rejected() {
        let mut memory = LinearMemory::new(0, 8);
        assert_eq!(memory.write_u64(1, 7), Err(MemoryError::OutOfBounds));
    }
}
