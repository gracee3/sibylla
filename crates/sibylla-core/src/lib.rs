//! Validated, deck-independent tarot domain contracts.
//!
//! This crate owns tarot values and serialization. It intentionally contains
//! no database, encryption, UI, astrology, image-recognition, or AI provider.

mod error;
mod id;
mod identity;
mod manifest;

pub use error::ValidationError;
pub use id::StableId;
pub use identity::{Arcana, CardIdentity, ConventionalCard, MajorArcana, MinorRank, MinorSuit};
pub use manifest::{
    Attribution, Correspondence, DRAW_MANIFEST_SCHEMA_VERSION, DeckCard, DeckManifest,
    MANIFEST_SCHEMA_VERSION, ManifestError, ReversalPolicy, RightsMetadata,
};
