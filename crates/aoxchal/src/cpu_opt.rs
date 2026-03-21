/// CPU capability profile used to drive deterministic policy decisions for
/// optional accelerated execution paths.
///
/// Security and operational notes:
/// - This type does not itself enable any unsafe fast path.
/// - It only reports capability signals that higher-level components may use
///   when selecting an implementation.
/// - The reporting model is intentionally small to reduce ambiguity and keep
///   the policy surface auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuCapabilities {
    /// Intel/AMD AES instruction support.
    pub aes_ni: bool,

    /// AVX2 vector extension support.
    pub avx2: bool,

    /// AVX-512 Foundation support.
    pub avx512f: bool,
}

impl CpuCapabilities {
    /// Returns a conservative portable profile with all optional hardware
    /// acceleration flags disabled.
    #[must_use]
    pub const fn portable() -> Self {
        Self {
            aes_ni: false,
            avx2: false,
            avx512f: false,
        }
    }

    /// Constructs a capability profile from explicit flags.
    ///
    /// This constructor is primarily useful for deterministic tests and policy
    /// evaluation code.
    #[must_use]
    pub const fn from_flags(aes_ni: bool, avx2: bool, avx512f: bool) -> Self {
        Self {
            aes_ni,
            avx2,
            avx512f,
        }
    }

    /// Detects supported CPU flags using architecture-aware runtime feature
    /// discovery where available.
    ///
    /// On non-x86 targets the function intentionally returns the conservative
    /// portable profile because the current capability model is x86-oriented.
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                aes_ni: std::is_x86_feature_detected!("aes"),
                avx2: std::is_x86_feature_detected!("avx2"),
                avx512f: std::is_x86_feature_detected!("avx512f"),
            }
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            Self::portable()
        }
    }

    /// Returns a deterministic profile label suitable for logs, manifests, and
    /// policy gates.
    ///
    /// The naming contract is intentionally stable and compact.
    #[must_use]
    pub const fn profile_name(self) -> &'static str {
        match (self.aes_ni, self.avx2, self.avx512f) {
            (true, true, true) => "aes-ni+avx2+avx512",
            (true, true, false) => "aes-ni+avx2",
            (true, false, _) => "aes-ni",
            _ => "portable",
        }
    }

    /// Returns `true` when the host can support AES-specific accelerated code
    /// paths.
    ///
    /// This method expresses the policy-relevant semantic rather than forcing
    /// callers to repeatedly inspect raw flags.
    #[must_use]
    pub const fn supports_accelerated_aead(self) -> bool {
        self.aes_ni
    }

    /// Returns `true` when the host can support wide-vector execution paths
    /// that benefit from AVX-class parallelism.
    #[must_use]
    pub const fn supports_wide_parallelism(self) -> bool {
        self.avx2 || self.avx512f
    }

    /// Returns `true` when the profile is fully portable and does not rely on
    /// optional x86 acceleration features.
    #[must_use]
    pub const fn is_portable(self) -> bool {
        !self.aes_ni && !self.avx2 && !self.avx512f
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
        let caps = CpuCapabilities::from_flags(true, false, false);

        assert_eq!(caps.profile_name(), "aes-ni");
        assert!(caps.supports_accelerated_aead());
        assert!(!caps.supports_wide_parallelism());
        assert!(!caps.is_portable());
    }

    #[test]
    fn aes_and_avx2_profile_name_is_deterministic() {
        let caps = CpuCapabilities::from_flags(true, true, false);

        assert_eq!(caps.profile_name(), "aes-ni+avx2");
        assert!(caps.supports_accelerated_aead());
        assert!(caps.supports_wide_parallelism());
    }

    #[test]
    fn full_profile_name_is_deterministic() {
        let caps = CpuCapabilities::from_flags(true, true, true);

        assert_eq!(caps.profile_name(), "aes-ni+avx2+avx512");
        assert!(caps.supports_accelerated_aead());
        assert!(caps.supports_wide_parallelism());
    }

    #[test]
    fn avx_without_aes_falls_back_to_portable_label_by_policy() {
        let caps = CpuCapabilities::from_flags(false, true, true);

        assert_eq!(caps.profile_name(), "portable");
        assert!(!caps.supports_accelerated_aead());
        assert!(caps.supports_wide_parallelism());
        assert!(!caps.is_portable());
    }

    #[test]
    fn detect_produces_a_self_consistent_profile() {
        let caps = CpuCapabilities::detect();
        let profile = caps.profile_name();

        assert!(matches!(
            profile,
            "portable" | "aes-ni" | "aes-ni+avx2" | "aes-ni+avx2+avx512"
        ));

        if caps.avx512f {
            assert!(caps.supports_wide_parallelism());
        }

        if caps.aes_ni {
            assert!(caps.supports_accelerated_aead());
        }
    }
}
