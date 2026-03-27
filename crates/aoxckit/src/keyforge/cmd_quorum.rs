// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{QuorumCommand, QuorumSubcommand};

pub fn handle(command: QuorumCommand) -> Result<(), String> {
    match command.command {
        QuorumSubcommand::Evaluate {
            total,
            approvals,
            threshold_bps,
        } => evaluate(total, approvals, threshold_bps),
    }
}

fn evaluate(total: u16, approvals: u16, threshold_bps: u16) -> Result<(), String> {
    if total == 0 {
        return Err("QUORUM_TOTAL_ZERO".to_string());
    }

    if approvals > total {
        return Err("QUORUM_APPROVALS_EXCEED_TOTAL".to_string());
    }

    if threshold_bps == 0 || threshold_bps > 10_000 {
        return Err("QUORUM_THRESHOLD_INVALID".to_string());
    }

    let required = required_approvals(total, threshold_bps);
    let passed = approvals >= required;

    let output = serde_json::json!({
        "total": total,
        "approvals": approvals,
        "threshold_bps": threshold_bps,
        "required_approvals": required,
        "passed": passed,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}

fn required_approvals(total: u16, threshold_bps: u16) -> u16 {
    let total = u32::from(total);
    let threshold_bps = u32::from(threshold_bps);
    let required = (total.saturating_mul(threshold_bps).saturating_add(9_999)) / 10_000;
    required as u16
}

#[cfg(test)]
mod tests {
    use super::required_approvals;

    #[test]
    fn required_approvals_rounds_up() {
        assert_eq!(required_approvals(10, 6667), 7);
        assert_eq!(required_approvals(3, 6667), 3);
    }

    #[test]
    fn required_approvals_handles_full_threshold() {
        assert_eq!(required_approvals(9, 10_000), 9);
    }
}
