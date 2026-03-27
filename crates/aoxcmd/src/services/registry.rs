// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ServiceDescriptor {
    pub name: &'static str,
    pub purpose: &'static str,
}

pub fn default_registry() -> Vec<ServiceDescriptor> {
    vec![
        ServiceDescriptor {
            name: "config",
            purpose: "Operator configuration resolution and validation",
        },
        ServiceDescriptor {
            name: "keys",
            purpose: "Identity material bootstrap and verification",
        },
        ServiceDescriptor {
            name: "node",
            purpose: "Local runtime state and block production lifecycle",
        },
        ServiceDescriptor {
            name: "economy",
            purpose: "Treasury and delegation state management",
        },
        ServiceDescriptor {
            name: "telemetry",
            purpose: "Metrics and operator diagnostics capture",
        },
    ]
}
