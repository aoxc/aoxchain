use thiserror::Error;

/// Upper allocation bound for a single managed region.
///
/// The value is intentionally conservative for a generic utility crate. It
/// protects the process against accidental oversized allocations in paths that
/// are not expected to manage unbounded data volumes.
///
/// Higher-level components may introduce specialized allocators or mapped
/// storage backends when requirements become explicit.
pub const DEFAULT_MAX_REGION_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

/// Errors emitted by the bounded memory-region facade.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryRegionError {
    /// Returned when the caller requests an empty region.
    #[error("memory region size must be greater than zero")]
    ZeroSizedRegion,

    /// Returned when the requested allocation exceeds the configured maximum.
    #[error("requested memory region size {requested} exceeds configured maximum {max}")]
    RegionTooLarge { requested: usize, max: usize },

    /// Returned when a caller attempts to read or write beyond the current
    /// region boundary.
    #[error(
        "memory access out of bounds: offset={offset}, length={length}, region_length={region_length}"
    )]
    OutOfBounds {
        offset: usize,
        length: usize,
        region_length: usize,
    },

    /// Returned when arithmetic required for bounds validation overflows.
    #[error("arithmetic overflow while validating memory region access")]
    ArithmeticOverflow,
}

/// Deterministic, bounded, zero-initialized memory region.
///
/// Design principles:
/// - Explicit allocation limits
/// - Zero-initialized construction
/// - Checked read/write operations
/// - No implicit growth semantics
/// - No unsafe code
///
/// This type is intentionally a strict in-memory region abstraction rather than
/// a full allocator or virtual memory subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRegion {
    data: Box<[u8]>,
}

impl MemoryRegion {
    /// Constructs a zero-initialized memory region using the default maximum
    /// region bound.
    pub fn new_zeroed(size: usize) -> Result<Self, MemoryRegionError> {
        Self::new_zeroed_bounded(size, DEFAULT_MAX_REGION_BYTES)
    }

    /// Constructs a zero-initialized memory region using an explicit maximum
    /// bound supplied by the caller.
    pub fn new_zeroed_bounded(size: usize, max_size: usize) -> Result<Self, MemoryRegionError> {
        if size == 0 {
            return Err(MemoryRegionError::ZeroSizedRegion);
        }

        if size > max_size {
            return Err(MemoryRegionError::RegionTooLarge {
                requested: size,
                max: max_size,
            });
        }

        let data = vec![0_u8; size].into_boxed_slice();
        Ok(Self { data })
    }

    /// Returns the region length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` when the region contains no bytes.
    ///
    /// Under the current constructor policy this will always be `false` for
    /// successfully constructed instances, but the method is kept because it is
    /// part of the standard slice-like API surface and simplifies generic use.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Exposes the full region as an immutable slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Exposes the full region as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Reads a checked sub-slice from the region.
    pub fn read(&self, offset: usize, length: usize) -> Result<&[u8], MemoryRegionError> {
        let end = offset
            .checked_add(length)
            .ok_or(MemoryRegionError::ArithmeticOverflow)?;

        if end > self.data.len() {
            return Err(MemoryRegionError::OutOfBounds {
                offset,
                length,
                region_length: self.data.len(),
            });
        }

        Ok(&self.data[offset..end])
    }

    /// Writes the provided bytes into the region at the specified offset.
    pub fn write(&mut self, offset: usize, input: &[u8]) -> Result<(), MemoryRegionError> {
        let length = input.len();
        let end = offset
            .checked_add(length)
            .ok_or(MemoryRegionError::ArithmeticOverflow)?;

        if end > self.data.len() {
            return Err(MemoryRegionError::OutOfBounds {
                offset,
                length,
                region_length: self.data.len(),
            });
        }

        self.data[offset..end].copy_from_slice(input);
        Ok(())
    }

    /// Fills the entire region with zero bytes.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Fills the entire region with the provided byte value.
    pub fn fill(&mut self, value: u8) {
        self.data.fill(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_MAX_REGION_BYTES, MemoryRegion, MemoryRegionError};

    #[test]
    fn zero_sized_region_is_rejected() {
        let result = MemoryRegion::new_zeroed(0);

        assert_eq!(result, Err(MemoryRegionError::ZeroSizedRegion));
    }

    #[test]
    fn oversized_region_is_rejected() {
        let result = MemoryRegion::new_zeroed_bounded(33, 32);

        assert_eq!(
            result,
            Err(MemoryRegionError::RegionTooLarge {
                requested: 33,
                max: 32
            })
        );
    }

    #[test]
    fn region_is_zero_initialized() {
        let region = MemoryRegion::new_zeroed(16).expect("region creation must succeed");

        assert_eq!(region.len(), 16);
        assert!(!region.is_empty());
        assert!(region.as_slice().iter().all(|byte| *byte == 0));
    }

    #[test]
    fn write_and_read_round_trip_is_lossless() {
        let mut region = MemoryRegion::new_zeroed(16).expect("region creation must succeed");
        let payload = [1_u8, 2, 3, 4];

        region
            .write(4, &payload)
            .expect("write within bounds must succeed");

        let view = region.read(4, payload.len()).expect("read must succeed");
        assert_eq!(view, payload.as_slice());
    }

    #[test]
    fn out_of_bounds_write_is_rejected() {
        let mut region = MemoryRegion::new_zeroed(8).expect("region creation must succeed");

        let error = region
            .write(7, &[1, 2])
            .expect_err("write must fail when it exceeds region boundary");

        assert_eq!(
            error,
            MemoryRegionError::OutOfBounds {
                offset: 7,
                length: 2,
                region_length: 8
            }
        );
    }

    #[test]
    fn out_of_bounds_read_is_rejected() {
        let region = MemoryRegion::new_zeroed(8).expect("region creation must succeed");

        let error = region
            .read(6, 3)
            .expect_err("read must fail when it exceeds region boundary");

        assert_eq!(
            error,
            MemoryRegionError::OutOfBounds {
                offset: 6,
                length: 3,
                region_length: 8
            }
        );
    }

    #[test]
    fn arithmetic_overflow_is_rejected_for_read() {
        let region = MemoryRegion::new_zeroed(8).expect("region creation must succeed");

        let error = region
            .read(usize::MAX, 1)
            .expect_err("overflowing access must fail");

        assert_eq!(error, MemoryRegionError::ArithmeticOverflow);
    }

    #[test]
    fn arithmetic_overflow_is_rejected_for_write() {
        let mut region = MemoryRegion::new_zeroed(8).expect("region creation must succeed");

        let error = region
            .write(usize::MAX, &[1])
            .expect_err("overflowing access must fail");

        assert_eq!(error, MemoryRegionError::ArithmeticOverflow);
    }

    #[test]
    fn clear_restores_region_to_zero_state() {
        let mut region = MemoryRegion::new_zeroed(8).expect("region creation must succeed");

        region.fill(0xAB);
        region.clear();

        assert!(region.as_slice().iter().all(|byte| *byte == 0));
    }

    #[test]
    fn explicit_default_bound_is_respected() {
        let region =
            MemoryRegion::new_zeroed_bounded(DEFAULT_MAX_REGION_BYTES, DEFAULT_MAX_REGION_BYTES)
                .expect("allocation at the exact limit must succeed");

        assert_eq!(region.len(), DEFAULT_MAX_REGION_BYTES);
    }
}
