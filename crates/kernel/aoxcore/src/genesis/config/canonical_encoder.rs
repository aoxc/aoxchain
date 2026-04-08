/// Canonical encoder for deterministic genesis fingerprints.
///
/// Security rationale:
/// This encoder provides explicit field framing and stable byte ordering.
/// Consensus-sensitive fingerprints must not depend on debug rendering,
/// map ordering, or serializer implementation details.
struct CanonicalEncoder {
    hasher: Sha256,
}

impl CanonicalEncoder {
    fn new(domain: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(domain);
        hasher.update([0x00]);
        Self { hasher }
    }

    fn u8(&mut self, value: u8) {
        self.hasher.update([value]);
    }

    fn u16(&mut self, value: u16) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.hasher.update(value.to_le_bytes());
    }

    fn u128(&mut self, value: u128) {
        self.hasher.update(value.to_le_bytes());
    }

    fn usize(&mut self, value: usize) -> Result<(), GenesisConfigError> {
        let casted = u64::try_from(value)
            .map_err(|_| GenesisConfigError::CanonicalEncodingLengthOverflow)?;
        self.u64(casted);
        Ok(())
    }

    fn str(&mut self, value: &str) {
        let len = u64::try_from(value.len()).unwrap_or(0);
        self.hasher.update(len.to_le_bytes());
        self.hasher.update(value.as_bytes());
    }

    fn strs(&mut self, values: &[String]) -> Result<(), GenesisConfigError> {
        self.usize(values.len())?;
        for value in values {
            self.str(value);
        }
        Ok(())
    }

    fn finish(self) -> [u8; 32] {
        let digest = self.hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }
}
