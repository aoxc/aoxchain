use super::*;

pub(in crate::cli::ops) fn effective_settings_for_ops() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

pub(in crate::cli::ops) fn parse_positive_u64_arg(
    args: &[String],
    flag: &str,
    default: u64,
    context: &str,
) -> Result<u64, AppError> {
    let value = match arg_value(args, flag) {
        Some(value) => normalize_text(&value, false).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank for {context}"),
            )
        })?,
        None => default.to_string(),
    };

    let parsed = value.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

pub(in crate::cli::ops) fn parse_positive_u64_value(
    value: &str,
    flag: &str,
    context: &str,
) -> Result<u64, AppError> {
    let normalized = normalize_text(value, false).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank for {context}"),
        )
    })?;

    let parsed = normalized.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

pub(in crate::cli::ops) fn parse_required_or_default_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
    lowercase: bool,
) -> Result<String, AppError> {
    match arg_value(args, flag) {
        Some(value) => normalize_text(&value, lowercase).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank"),
            )
        }),
        None => Ok(default.to_string()),
    }
}

pub(in crate::cli::ops) fn parse_optional_text_arg(
    args: &[String],
    flag: &str,
    lowercase: bool,
) -> Option<String> {
    arg_value(args, flag).and_then(|value| normalize_text(&value, lowercase))
}

pub(in crate::cli::ops) fn normalize_text(value: &str, lowercase: bool) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    if lowercase {
        Some(normalized.to_ascii_lowercase())
    } else {
        Some(normalized)
    }
}
