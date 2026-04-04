mod code;
mod constants;
mod entry;
mod enums;
mod error;
mod identifiers;

pub use constants::{ASSET_CODE_NAMESPACE, ASSET_CODE_SEQUENCE_LEN, MAX_DECIMALS};
pub use entry::AssetRegistryEntry;
pub use enums::{AssetClass, MintAuthority, RegistryStatus, RiskGrade, SupplyModel};
pub use error::AssetRegistryError;
pub use identifiers::{AssetCode, AssetId, AssetSymbol, DisplayName, IssuerId, NonZeroHash32};

#[cfg(test)]
mod tests;
