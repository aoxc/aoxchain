use super::*;

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = parse_required_or_default_text_arg(args, "--to", "ops", false)?;
    let amount = parse_positive_u64_arg(args, "--amount", 1000, "treasury transfer")?;

    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
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
