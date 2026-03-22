//! Contract RPC subsystem.
//!
//! This module defines the transport-facing request/response boundary for the
//! contract control plane. It does not derive identities, duplicate semantic
//! validation, or implement registry/runtime logic itself; instead it delegates
//! to `aoxcontract`, `aoxcore`, `aoxcdata`, and `aoxcvm`.

pub mod error;
pub mod http;
pub mod mapper;
pub mod read_model;
pub mod service;
pub mod types;
pub mod validation;
pub mod write_model;

pub use error::ContractRpcError;
pub use http::ContractHttpApi;
pub use read_model::{ContractDetailView, ContractRuntimeBindingView, ContractSummaryView};
pub use service::{
    ContractCommandService, ContractQueryService, ContractRuntimeBindingService,
    ContractValidationService,
};
pub use types::*;
