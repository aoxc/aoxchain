use serde::{Deserialize, Serialize};

/// CPU capability profile used to drive deterministic policy decisions for
/// optional accelerated execution paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CpuCapabilities {
    /// Hardware AES instruction support (Intel AES-NI or ARM AES).
    pub aes_hw: bool,

    /// AVX2 vector extension support (x86 only).
    pub avx2: bool,

    /// AVX-512 Foundation support (x86 only).
    pub avx512f: bool,

    /// NEON vector extension support (ARM only).
    pub neon: bool,
}

impl CpuCapabilities {
    /// Returns a conservative portable profile with all optional hardware
    /// acceleration flags disabled.
    #[must_use]
    pub const fn portable() -> Self {
        Self {
            aes_hw: false,
            avx2: false,
            avx512f: false,
            neon: false,
        }
    }

    /// Constructs a capability profile from explicit flags.
    #[must_use]
    pub const fn from_flags(aes_hw: bool, avx2: bool, avx512f: bool, neon: bool) -> Self {
        Self {
            aes_hw,
            avx2,
            avx512f,
            neon,
        }
    }

    /// Detects supported CPU flags using architecture-aware runtime feature discovery.
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                aes_hw: std::is_x86_feature_detected!("aes"),
                avx2: std::is_x86_feature_detected!("avx2"),
                avx512f: std::is_x86_feature_detected!("avx512f"),
                neon: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self {
                aes_hw: std::arch::is_aarch64_feature_detected!("aes"),
                avx2: false,
                avx512f: false,
                neon: std::arch::is_aarch64_feature_detected!("neon"),
            }
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self::portable()
        }
    }

    /// Returns a deterministic profile label suitable for logs, manifests, and policy gates.
    #[must_use]
    pub const fn profile_name(self) -> &'static str {
        if self.is_portable() {
            return "portable";
        }

        match (self.aes_hw, self.avx512f, self.avx2, self.neon) {
            (true, true, _, _) => "aes-hw+avx512",
            (true, false, true, _) => "aes-hw+avx2",
            (true, false, false, true) => "aes-hw+neon",
            (true, false, false, false) => "aes-hw",
            (false, true, _, _) => "avx512",
            (false, false, true, _) => "avx2",
            (false, false, false, true) => "neon",
            _ => "portable",
        }
    }

    /// Returns `true` when the host can support AES-specific accelerated code paths.
    #[must_use]
    pub const fn supports_accelerated_aead(self) -> bool {
        self.aes_hw
    }

    /// Returns `true` when the host can support wide-vector execution paths.
    #[must_use]
    pub const fn supports_wide_parallelism(self) -> bool {
        self.avx2 || self.avx512f || self.neon
    }

    /// Returns `true` when the profile is fully portable and does not rely on any hardware acceleration.
    #[must_use]
    pub const fn is_portable(self) -> bool {
        !self.aes_hw && !self.avx2 && !self.avx512f && !self.neon
    }
}

#[cfg(test)]
mod tests {
    use super::CpuCapabilities;

    #[test]
    fn portable_profile_is_reported_correctly() {
        let caps = CpuCapabilities::portable();
        assert!(caps.is_portable());
        assert_eq!(caps.profile_name(), "portable");
        assert!(!caps.supports_accelerated_aead());
        assert!(!caps.supports_wide_parallelism());
    }

    #[test]
    fn aes_only_profile_name_is_deterministic() {
        let caps = CpuCapabilities::from_flags(true, false, false, false);
        assert_eq!(caps.profile_name(), "aes-hw");
        assert!(caps.supports_accelerated_aead());
        assert!(!caps.is_portable());
    }

    #[test]
    fn aes_and_avx2_profile_name_is_deterministic() {
        let caps = CpuCapabilities::from_flags(true, true, false, false);
        assert_eq!(caps.profile_name(), "aes-hw+avx2");
        assert!(caps.supports_accelerated_aead());
        assert!(caps.supports_wide_parallelism());
    }

    #[test]
    fn avx_without_aes_now_reports_correctly() {
        let caps = CpuCapabilities::from_flags(false, true, false, false);
        assert_eq!(caps.profile_name(), "avx2"); // Eskiden hatalı olarak "portable" dönüyordu
        assert!(!caps.is_portable()); // Artık tutarlı!
    }

    #[test]
    fn arm_neon_is_supported_and_deterministic() {
        let caps = CpuCapabilities::from_flags(true, false, false, true);
        assert_eq!(caps.profile_name(), "aes-hw+neon");
        assert!(caps.supports_wide_parallelism());
        assert!(!caps.is_portable());
    }
}
