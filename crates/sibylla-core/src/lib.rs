//! Validated, deck-independent tarot domain contracts.
//!
//! This crate owns tarot values and serialization. It intentionally contains
//! no database, encryption, UI, astrology, image-recognition, or AI provider.

mod error;
mod id;
mod identity;
mod manifest;
mod reading;
mod spread;
mod time;
mod validation;

pub use error::ValidationError;
pub use id::StableId;
pub use identity::{Arcana, CardIdentity, ConventionalCard, MajorArcana, MinorRank, MinorSuit};
pub use manifest::{
    Attribution, Correspondence, DRAW_MANIFEST_SCHEMA_VERSION, DeckCard, DeckManifest,
    MANIFEST_SCHEMA_VERSION, ManifestError, ReversalPolicy, RightsMetadata,
};
pub use reading::{
    DrawProvenance, FollowUp, FollowUpKind, Orientation, Placement, READING_SCHEMA_VERSION,
    ReadingError, TarotReading,
};
pub use spread::{
    LayoutHint, SPREAD_SCHEMA_VERSION, SpreadDefinition, SpreadError, SpreadLayout, SpreadPosition,
};
pub use time::UtcInstant;
