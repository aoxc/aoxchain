//! Reference adapter model for integrating `aoxcai` into kernel-adjacent crates.
//!
//! Adapters are the only approved integration pattern:
//! `crate -> adapter -> aoxcai -> policy -> authorization -> audit`.
//! Adapters must declare zone, capability, and action class explicitly and must
//! never hide audit or perform autonomous state mutation.

use serde::{Deserialize, Serialize};

use crate::{AiActionClass, AiCapability, KernelZone};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterInvocation {
    pub caller_crate: String,
    pub caller_component: String,
    pub requested_action: String,
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
}
