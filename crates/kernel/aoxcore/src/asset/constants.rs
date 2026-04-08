/// Maximum supported decimal precision for registry assets.
///
/// Rationale:
/// The registry deliberately caps decimal precision to reduce downstream
/// formatting ambiguity and avoid excessive precision assumptions across
/// execution, accounting, and client surfaces.
pub const MAX_DECIMALS: u8 = 18;

/// Required namespace prefix for canonical AOXC asset codes.
pub const ASSET_CODE_NAMESPACE: &str = "AOXC";

/// Canonical length of the sequence segment in the asset code.
pub const ASSET_CODE_SEQUENCE_LEN: usize = 4;
