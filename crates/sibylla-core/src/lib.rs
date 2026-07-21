//! Validated, deck-independent tarot domain contracts.
//!
//! This crate owns tarot values and serialization. It intentionally contains
//! no database, encryption, UI, astrology, image-recognition, or AI provider.

mod error;
mod id;

pub use error::ValidationError;
pub use id::StableId;
