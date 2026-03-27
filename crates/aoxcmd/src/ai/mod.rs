// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXCMD integration layer for the optional `aoxcai` subsystem.
//!
//! The operator command plane is currently the only real integration target for
//! AI assistance inside the workspace. This module therefore remains narrowly
//! scoped to operator-facing, non-authoritative workflows such as diagnostics
//! explanation and guarded runbook preparation.
//!
//! Native AOXCMD verdicts remain authoritative. AI output produced through this
//! module is advisory or guarded-preparation only, never canonical truth and
//! never an automatic execution surface.

pub mod context;
pub mod operator;
pub mod runtime;
pub mod signals;
