use serde::Serialize;

pub const AOXC_COVENANT_KERNEL_NAME: &str = "AOXC Covenant Kernel";
pub const AOXC_COVENANT_KERNEL_LINE: &str = "AOXC-COVENANT-KERNEL-V1-draft";
pub const AOXC_VOTE_FORMAT_LINE: &str = "AOXC-VOTE-FMT-V1-draft";
pub const AOXC_CERTIFICATE_FORMAT_LINE: &str = "AOXC-CERT-FMT-V1-draft";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct KernelIdentity {
    pub name: &'static str,
    pub line: &'static str,
    pub vote_format: &'static str,
    pub certificate_format: &'static str,
}

#[must_use]
pub const fn kernel_identity() -> KernelIdentity {
    KernelIdentity {
        name: AOXC_COVENANT_KERNEL_NAME,
        line: AOXC_COVENANT_KERNEL_LINE,
        vote_format: AOXC_VOTE_FORMAT_LINE,
        certificate_format: AOXC_CERTIFICATE_FORMAT_LINE,
    }
}
