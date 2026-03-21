/// Logical column families used by the AOXC data layer.
///
/// The enum is intentionally stable because it forms part of the storage path
/// namespace for the filesystem-backed KV surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColumnFamily {
    Blocks,
    Transactions,
    Receipts,
    State,
    Metadata,
}

impl ColumnFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::Transactions => "transactions",
            Self::Receipts => "receipts",
            Self::State => "state",
            Self::Metadata => "metadata",
        }
    }
}

#[must_use]
pub const fn all_column_families() -> [ColumnFamily; 5] {
    [
        ColumnFamily::Blocks,
        ColumnFamily::Transactions,
        ColumnFamily::Receipts,
        ColumnFamily::State,
        ColumnFamily::Metadata,
    ]
}
