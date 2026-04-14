use super::*;

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TransferSecurityMode {
    Full,
    FullMultisig,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct TransferAuthorization {
    mode: TransferSecurityMode,
    signature_scheme: String,
    threshold: u16,
    signers: Vec<String>,
    signature_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct TransferExecution {
    to: String,
    amount: u64,
    authorization: TransferAuthorization,
    ledger: ledger::LedgerState,
}

fn transfer_authorization(args: &[String]) -> Result<TransferAuthorization, AppError> {
    if has_flag(args, "--allow-unsigned") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "--allow-unsigned is not permitted in production-grade transfer flow",
        ));
    }

    let signature_scheme =
        parse_required_or_default_text_arg(args, "--signature-scheme", "ed25519", true)?;
    let threshold = parse_multisig_threshold(args)?;
    let signers = parse_comma_separated_text_arg(args, "--signers");
    let signatures = parse_comma_separated_text_arg(args, "--signatures");
    let single_signature = parse_optional_text_arg(args, "--signature", false);

    if threshold.is_some() || !signers.is_empty() || !signatures.is_empty() {
        return parse_multisig_authorization(signature_scheme, threshold, signers, signatures);
    }

    let Some(signature) = single_signature else {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Transfer requires --signature or multisig set (--multisig-threshold/--signers/--signatures)",
        ));
    };

    if signature.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --signature must not be blank",
        ));
    }

    Ok(TransferAuthorization {
        mode: TransferSecurityMode::Full,
        signature_scheme,
        threshold: 1,
        signers: vec!["single-signer".to_string()],
        signature_count: 1,
    })
}

fn parse_multisig_authorization(
    signature_scheme: String,
    threshold: Option<u16>,
    signers: Vec<String>,
    signatures: Vec<String>,
) -> Result<TransferAuthorization, AppError> {
    let threshold = threshold.ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Multisig transfer requires --multisig-threshold",
        )
    })?;

    if threshold < 2 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --multisig-threshold must be at least 2",
        ));
    }

    if signers.is_empty() || signatures.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Multisig transfer requires non-empty --signers and --signatures lists",
        ));
    }

    if signers.len() != signatures.len() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Multisig transfer requires equal signer and signature counts",
        ));
    }

    if threshold as usize > signatures.len() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Multisig threshold exceeds provided signatures",
        ));
    }

    Ok(TransferAuthorization {
        mode: TransferSecurityMode::FullMultisig,
        signature_scheme,
        threshold,
        signers,
        signature_count: signatures.len(),
    })
}

fn parse_multisig_threshold(args: &[String]) -> Result<Option<u16>, AppError> {
    let Some(raw) = parse_optional_text_arg(args, "--multisig-threshold", false) else {
        return Ok(None);
    };

    let parsed = raw.parse::<u16>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid numeric value for --multisig-threshold",
        )
    })?;

    Ok(Some(parsed))
}

fn parse_comma_separated_text_arg(args: &[String], flag: &str) -> Vec<String> {
    parse_optional_text_arg(args, flag, false)
        .map(|raw| {
            raw.split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = parse_required_or_default_text_arg(args, "--to", "ops", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "treasury transfer")?;
    let authorization = transfer_authorization(args)?;

    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    let execution = TransferExecution {
        to,
        amount,
        authorization,
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
    use super::{TransferSecurityMode, transfer_authorization};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn transfer_authorization_rejects_allow_unsigned() {
        let error = transfer_authorization(&args(&["--allow-unsigned"]))
            .expect_err("production flow should reject unsigned overrides");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn transfer_authorization_accepts_single_signature_flow() {
        let authorization = transfer_authorization(&args(&[
            "--to",
            "ops",
            "--amount",
            "10",
            "--signature",
            "deadbeef",
        ]))
        .expect("signed transfer should pass");
        assert_eq!(authorization.mode, TransferSecurityMode::Full);
        assert_eq!(authorization.threshold, 1);
        assert_eq!(authorization.signature_count, 1);
    }

    #[test]
    fn transfer_authorization_accepts_multisig_flow() {
        let authorization = transfer_authorization(&args(&[
            "--multisig-threshold",
            "2",
            "--signers",
            "alice,bob,carol",
            "--signatures",
            "sig-a,sig-b,sig-c",
        ]))
        .expect("multisig transfer should pass");
        assert_eq!(authorization.mode, TransferSecurityMode::FullMultisig);
        assert_eq!(authorization.threshold, 2);
        assert_eq!(authorization.signature_count, 3);
        assert_eq!(authorization.signers.len(), 3);
    }

    #[test]
    fn transfer_authorization_rejects_multisig_threshold_above_signature_count() {
        let error = transfer_authorization(&args(&[
            "--multisig-threshold",
            "3",
            "--signers",
            "alice,bob",
            "--signatures",
            "sig-a,sig-b",
        ]))
        .expect_err("multisig threshold above signer/sig count must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }
}
