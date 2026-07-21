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
    #[error("unknown conventional card identity {0}")]
    UnknownConventionalCard(String),
    #[error("reversal rate must be in 0..=10000 basis points, got {0}")]
    InvalidReversalRate(u16),
    #[error("a deck manifest must contain at least one card")]
    EmptyDeck,
    #[error("a deck manifest must enable at least one card")]
    NoEnabledCards,
    #[error("deck card IDs must be unique: {0}")]
    DuplicateDeckCardId(crate::StableId),
    #[error("correspondence keys must be unique: {0}")]
    DuplicateCorrespondenceKey(crate::StableId),
}
