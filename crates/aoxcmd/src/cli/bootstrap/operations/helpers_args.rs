use super::*;

pub(super) fn parse_required_text_arg(
    args: &[String],
    flag: &str,
    lowercase: bool,
    context: &str,
) -> Result<String, AppError> {
    let value = arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required flag {flag} for {context}"),
        )
    })?;

    normalize_text(&value, lowercase).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank"),
        )
    })
}

pub(super) fn parse_required_or_default_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
) -> Result<String, AppError> {
    match arg_value(args, flag) {
        Some(value) => normalize_text(&value, false).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank"),
            )
        }),
        None => Ok(default.to_string()),
    }
}

pub(super) fn parse_optional_text_arg(
    args: &[String],
    flag: &str,
    lowercase: bool,
) -> Option<String> {
    arg_value(args, flag).and_then(|value| normalize_text(&value, lowercase))
}

fn normalize_text(value: &str, lowercase: bool) -> Option<String> {
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

pub(super) fn is_decimal_string(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch.is_ascii_digit())
}

pub(super) fn is_non_zero_decimal_string(value: &str) -> bool {
    is_decimal_string(value) && value.trim().chars().any(|ch| ch != '0')
}
