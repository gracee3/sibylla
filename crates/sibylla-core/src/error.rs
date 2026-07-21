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
    #[error("{field} must be finite")]
    NonFinite { field: &'static str },
    #[error("a spread must contain at least one position")]
    EmptySpread,
    #[error("spread position IDs must be unique: {0}")]
    DuplicateSpreadPositionId(crate::StableId),
    #[error("a reading must contain at least one placement")]
    EmptyPlacementSet,
    #[error("placement position IDs must be unique: {0}")]
    DuplicatePlacementPositionId(crate::StableId),
    #[error("a physical deck card may appear only once in a reading: {0}")]
    DuplicatePlacedDeckCard(crate::StableId),
    #[error("draw orders must be the contiguous one-based range 1..=placement count")]
    InvalidDrawOrder,
    #[error("placement references unknown spread position {0}")]
    UnknownSpreadPosition(crate::StableId),
    #[error("placement label does not match spread position {0}")]
    PositionLabelMismatch(crate::StableId),
    #[error("placement references unknown deck card {0}")]
    UnknownDeckCard(crate::StableId),
    #[error("placement references disabled deck card {0}")]
    DisabledDeckCard(crate::StableId),
    #[error("placement identity does not match deck card {0}")]
    CardIdentityMismatch(crate::StableId),
    #[error("placement printed title does not match deck card {0}")]
    PrintedTitleMismatch(crate::StableId),
    #[error("a fixed spread requires exactly one placement for every position")]
    IncompleteFixedSpread,
    #[error("follow-up IDs must be unique: {0}")]
    DuplicateFollowUpId(crate::StableId),
    #[error("{field} timestamp is outside the reading timeline")]
    InvalidTimestampOrder { field: &'static str },
    #[error("follow-ups must remain in nondecreasing timestamp order")]
    FollowUpOrder,
    #[error("invalid RFC 3339 timestamp: {0}")]
    InvalidUtcInstant(String),
}
