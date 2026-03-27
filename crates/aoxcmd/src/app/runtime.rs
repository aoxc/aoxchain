// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    economy::ledger,
    error::AppError,
    node::lifecycle,
    telemetry::prometheus::{now, persist_metrics},
};

pub fn refresh_runtime_metrics() -> Result<(), AppError> {
    let node_state = lifecycle::load_state()?;
    let ledger_state = ledger::load()?;
    let metrics = now(
        node_state.current_height,
        node_state.produced_blocks,
        ledger_state.treasury_balance,
    );
    persist_metrics(&metrics)?;
    Ok(())
}
