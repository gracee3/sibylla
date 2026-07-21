use thiserror::Error;

/// A malformed domain value rejected during construction or deserialization.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ValidationError {
    #[error("{field} must not be empty")]
    EmptyText { field: &'static str },
    #[error(
        "{field} must be a lowercase identifier beginning with a letter and containing only letters, digits, or underscores"
    )]
    InvalidStableId { field: &'static str },
}
