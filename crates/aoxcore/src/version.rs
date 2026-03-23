use serde::Serialize;

pub const AOXC_CANONICAL_CORE_NAME: &str = "AOXC Canonical Core";
pub const AOXC_CORE_LINE: &str = "AOXC-CORE-V1";
pub const AOXC_BLOCK_FORMAT_LINE: &str = "AOXC-BLOCK-FMT-V1-draft";
pub const AOXC_GENESIS_AUTH_LINE: &str = "AOXC-GENESIS-AUTH-V1-draft";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CoreIdentity {
    pub name: &'static str,
    pub line: &'static str,
    pub block_format: &'static str,
    pub genesis_authority: &'static str,
}

#[must_use]
pub const fn core_identity() -> CoreIdentity {
    CoreIdentity {
        name: AOXC_CANONICAL_CORE_NAME,
        line: AOXC_CORE_LINE,
        block_format: AOXC_BLOCK_FORMAT_LINE,
        genesis_authority: AOXC_GENESIS_AUTH_LINE,
    }
}
