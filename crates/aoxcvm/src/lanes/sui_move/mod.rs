//! Sui / Move-style object-centric execution lane.

pub mod executor;
pub mod object_store;
pub mod package_store;
pub mod receipt;

pub use executor::SuiMoveExecutor;
