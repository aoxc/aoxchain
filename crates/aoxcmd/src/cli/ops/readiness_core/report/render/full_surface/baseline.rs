use super::*;

pub(in crate::cli::ops) fn append_baseline_section(
    out: &mut String,
    title: &str,
    baseline: Option<&ProfileBaselineReport>,
) {
    out.push_str(&format!("### {}\n\n", title));
    match baseline {
        Some(report) => {
            out.push_str(&format!(
                "- Status: **{}**\n",
                if report.passed {
                    "aligned"
                } else {
                    "drift-detected"
                }
            ));
            out.push_str(&format!("- Mainnet file: `{}`\n", report.mainnet_path));
            out.push_str(&format!("- Testnet file: `{}`\n", report.testnet_path));
            for control in &report.shared_controls {
                out.push_str(&format!(
                    "- {}: {} (mainnet=`{}`, testnet=`{}`)\n",
                    control.name,
                    if control.passed { "ok" } else { "drift" },
                    control.mainnet,
                    control.testnet
                ));
            }
            if !report.drift.is_empty() {
                out.push_str("- Drift summary:\n");
                for drift in &report.drift {
                    out.push_str(&format!("  - {}\n", drift));
                }
            }
        }
        None => out.push_str("- Status: unavailable\n"),
    }
    out.push('\n');
}
