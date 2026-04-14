use super::*;

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TransferSecurityMode {
    Full,
    DevelopmentUnsigned,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct TransferExecution {
    to: String,
    amount: u64,
    signature_provided: bool,
    security_mode: TransferSecurityMode,
    ledger: ledger::LedgerState,
}

fn transfer_security_mode(args: &[String]) -> Result<(bool, TransferSecurityMode), AppError> {
    let signature = parse_optional_text_arg(args, "--signature", false);
    let allow_unsigned = has_flag(args, "--allow-unsigned");

    match (signature.is_some(), allow_unsigned) {
        (true, true) => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use either --signature or --allow-unsigned, not both",
        )),
        (true, false) => Ok((true, TransferSecurityMode::Full)),
        (false, true) => Ok((false, TransferSecurityMode::DevelopmentUnsigned)),
        (false, false) => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Transfer requires --signature for full security (or --allow-unsigned for local development)",
        )),
    }
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = parse_required_or_default_text_arg(args, "--to", "ops", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "treasury transfer")?;
    let (signature_provided, security_mode) = transfer_security_mode(args)?;

    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    let execution = TransferExecution {
        to,
        amount,
        signature_provided,
        security_mode,
        ledger,
    };
    emit_serialized(&execution, output_format(args))
}

pub fn cmd_stake_delegate(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_or_default_text_arg(args, "--validator", "validator-01", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "stake delegation")?;

    let ledger = ledger::delegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_undelegate(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_or_default_text_arg(args, "--validator", "validator-01", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "stake undelegation")?;

    let ledger = ledger::undelegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_economy_status(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::load()?;
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_runtime_status(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let handles = default_handles();
    let unity = unity_status();
    let ai = crate::ai::runtime::report();

    #[derive(serde::Serialize)]
    struct RuntimeStatus {
        context: crate::runtime::context::RuntimeContext,
        handles: crate::runtime::handles::RuntimeHandleSet,
        unity: crate::runtime::unity::UnityStatus,
        ai: crate::ai::runtime::AiRuntimeReport,
    }

    let status = RuntimeStatus {
        context,
        handles,
        unity,
        ai,
    };

    emit_serialized(&status, output_format(args))
}

#[cfg(test)]
mod tests {
    use super::{TransferSecurityMode, transfer_security_mode};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn transfer_security_mode_requires_signature_by_default() {
        let error = transfer_security_mode(&args(&["--to", "ops", "--amount", "10"]))
            .expect_err("missing signature must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn transfer_security_mode_accepts_signed_flow() {
        let (signature_provided, mode) = transfer_security_mode(&args(&[
            "--to",
            "ops",
            "--amount",
            "10",
            "--signature",
            "deadbeef",
        ]))
        .expect("signed transfer should pass");
        assert!(signature_provided);
        assert_eq!(mode, TransferSecurityMode::Full);
    }

    #[test]
    fn transfer_security_mode_accepts_explicit_unsigned_dev_flow() {
        let (signature_provided, mode) = transfer_security_mode(&args(&["--allow-unsigned"]))
            .expect("explicit dev override should pass");
        assert!(!signature_provided);
        assert_eq!(mode, TransferSecurityMode::DevelopmentUnsigned);
    }
}
