use crate::ValidationError;

pub(crate) fn validate_text(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::EmptyText { field })
    } else {
        Ok(())
    }
}

pub(crate) fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ValidationError> {
    if let Some(value) = value {
        validate_text(field, value)?;
    }
    Ok(())
}
