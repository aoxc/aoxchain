use super::*;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

const TRANSFER_SIGNATURE_DOMAIN: &str = "aoxc.treasury-transfer.v1";

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
    signer_public_key: Option<String>,
    security_mode: TransferSecurityMode,
    ledger: ledger::LedgerState,
}

fn transfer_security_mode(
    args: &[String],
    to: &str,
    amount: u64,
) -> Result<(bool, Option<String>, TransferSecurityMode), AppError> {
    let signature = parse_optional_text_arg(args, "--signature", false);
    let public_key = parse_optional_text_arg(args, "--public-key", false);
    let allow_unsigned = has_flag(args, "--allow-unsigned");

    match (signature, allow_unsigned) {
        (Some(_), true) => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use either --signature or --allow-unsigned, not both",
        )),
        (Some(signature), false) => {
            let signer_public_key = public_key.ok_or_else(|| {
                AppError::new(
                    ErrorCode::UsageInvalidArguments,
                    "Transfer requires --public-key when --signature is provided",
                )
            })?;
            validate_transfer_signature(&signature, &signer_public_key, to, amount)?;
            Ok((true, Some(signer_public_key), TransferSecurityMode::Full))
        }
        (None, true) => Ok((false, None, TransferSecurityMode::DevelopmentUnsigned)),
        (None, false) => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Transfer requires --signature and --public-key for full security (or --allow-unsigned for local development)",
        )),
    }
}

fn transfer_signing_payload(to: &str, amount: u64) -> String {
    format!("{TRANSFER_SIGNATURE_DOMAIN}:{to}:{amount}")
}

fn decode_signature_bytes(signature: &str) -> Result<[u8; 64], AppError> {
    let bytes = hex::decode(signature).map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --signature must be valid hex-encoded bytes",
        )
    })?;
    bytes.try_into().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --signature must decode to 64 bytes",
        )
    })
}

fn decode_public_key_bytes(public_key: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(public_key).map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --public-key must be valid hex-encoded bytes",
        )
    })?;
    bytes.try_into().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --public-key must decode to 32 bytes",
        )
    })
}

fn validate_transfer_signature(
    signature: &str,
    public_key: &str,
    to: &str,
    amount: u64,
) -> Result<(), AppError> {
    let signature = Signature::from_slice(&decode_signature_bytes(signature)?).map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --signature must decode into an Ed25519 signature",
        )
    })?;
    let verifying_key = VerifyingKey::from_bytes(&decode_public_key_bytes(public_key)?).map_err(
        |_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --public-key must decode into an Ed25519 verifying key",
            )
        },
    )?;
    let payload = transfer_signing_payload(to, amount);
    verifying_key
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Transfer signature verification failed for the provided --to and --amount",
            )
        })?;
    Ok(())
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = parse_required_or_default_text_arg(args, "--to", "ops", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "treasury transfer")?;
    let (signature_provided, signer_public_key, security_mode) =
        transfer_security_mode(args, &to, amount)?;

    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    let execution = TransferExecution {
        to,
        amount,
        signature_provided,
        signer_public_key,
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
    use super::{transfer_security_mode, transfer_signing_payload, TransferSecurityMode};
    use ed25519_dalek::{Signer, SigningKey};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn transfer_security_mode_requires_signature_by_default() {
        let error = transfer_security_mode(&args(&["--to", "ops", "--amount", "10"]), "ops", 10)
            .expect_err("missing signature must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn transfer_security_mode_accepts_signed_flow() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let verifying_hex = hex::encode(signing_key.verifying_key().to_bytes());
        let payload = transfer_signing_payload("ops", 10);
        let signature_hex = hex::encode(signing_key.sign(payload.as_bytes()).to_bytes());

        let (signature_provided, signer_public_key, mode) = transfer_security_mode(
            &args(&[
                "--to",
                "ops",
                "--amount",
                "10",
                "--public-key",
                &verifying_hex,
                "--signature",
                &signature_hex,
            ]),
            "ops",
            10,
        )
        .expect("signed transfer should pass");
        assert!(signature_provided);
        assert_eq!(signer_public_key, Some(verifying_hex));
        assert_eq!(mode, TransferSecurityMode::Full);
    }

    #[test]
    fn transfer_security_mode_rejects_missing_public_key() {
        let error = transfer_security_mode(
            &args(&["--to", "ops", "--amount", "10", "--signature", "deadbeef"]),
            "ops",
            10,
        )
        .expect_err("missing public key must fail");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn transfer_security_mode_rejects_invalid_signature_for_payload() {
        let signing_key = SigningKey::from_bytes(&[9u8; 32]);
        let verifying_hex = hex::encode(signing_key.verifying_key().to_bytes());
        let payload = transfer_signing_payload("ops", 10);
        let signature_hex = hex::encode(signing_key.sign(payload.as_bytes()).to_bytes());

        let error = transfer_security_mode(
            &args(&[
                "--to",
                "ops",
                "--amount",
                "11",
                "--public-key",
                &verifying_hex,
                "--signature",
                &signature_hex,
            ]),
            "ops",
            11,
        )
        .expect_err("signature must be bound to payload");
        assert_eq!(error.code(), "AOXC-USG-002");
    }

    #[test]
    fn transfer_security_mode_accepts_explicit_unsigned_dev_flow() {
        let (signature_provided, signer_public_key, mode) =
            transfer_security_mode(&args(&["--allow-unsigned"]), "ops", 10)
                .expect("explicit dev override should pass");
        assert!(!signature_provided);
        assert_eq!(signer_public_key, None);
        assert_eq!(mode, TransferSecurityMode::DevelopmentUnsigned);
    }

    #[test]
    fn transfer_security_mode_rejects_non_hex_signature() {
        let error = transfer_security_mode(
            &args(&["--to", "ops", "--amount", "10", "--signature", "not-hex"]),
            "ops",
            10,
        )
            .expect_err("invalid signature");
        assert_eq!(error.code(), "AOXC-USG-002");
    }
}
