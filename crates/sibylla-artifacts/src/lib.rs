//! Stable, content-addressed interchange envelopes for Sibylla domain values.
//!
//! This crate is deliberately storage- and transport-neutral. Encryption,
//! persistence, synchronization, and application concerns belong to consumers.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sibylla_core::{DeckManifest, Sha256Id, TarotReading};
use thiserror::Error;

/// The only artifact-envelope schema version understood by this release.
pub const ARTIFACT_SCHEMA_VERSION: u32 = 1;

/// A stable discriminator for artifact payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Deck,
    Reading,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Deck => "deck",
            Self::Reading => "reading",
        })
    }
}

/// A validated deck manifest in a versioned interchange envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckArtifact {
    payload: DeckManifest,
}

/// A validated tarot reading in a versioned interchange envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadingArtifact {
    payload: TarotReading,
}

/// Any supported Sibylla interchange artifact.
#[derive(Clone, Debug, PartialEq)]
pub enum Artifact {
    Deck(Box<DeckArtifact>),
    Reading(Box<ReadingArtifact>),
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("invalid artifact JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported artifact schema version {0}")]
    UnsupportedSchema(u32),
    #[error("expected {expected} artifact, found {actual}")]
    UnexpectedType {
        expected: ArtifactKind,
        actual: ArtifactKind,
    },
}

#[derive(Serialize)]
struct EnvelopeRef<'a, T> {
    schema_version: u32,
    artifact_type: ArtifactKind,
    payload: &'a T,
}

#[derive(Deserialize)]
#[serde(tag = "artifact_type", rename_all = "snake_case", deny_unknown_fields)]
enum EnvelopeWire {
    Deck {
        schema_version: u32,
        payload: Box<DeckManifest>,
    },
    Reading {
        schema_version: u32,
        payload: Box<TarotReading>,
    },
}

impl DeckArtifact {
    pub const fn new(payload: DeckManifest) -> Self {
        Self { payload }
    }

    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        match Artifact::from_json(input)? {
            Artifact::Deck(artifact) => Ok(*artifact),
            Artifact::Reading(_) => Err(ArtifactError::UnexpectedType {
                expected: ArtifactKind::Deck,
                actual: ArtifactKind::Reading,
            }),
        }
    }

    pub const fn payload(&self) -> &DeckManifest {
        &self.payload
    }

    pub fn into_payload(self) -> DeckManifest {
        self.payload
    }

    pub fn to_json(&self) -> Result<String, ArtifactError> {
        serialize_compact(ArtifactKind::Deck, &self.payload)
    }

    pub fn to_pretty_json(&self) -> Result<String, ArtifactError> {
        serialize_pretty(ArtifactKind::Deck, &self.payload)
    }

    pub fn content_id(&self) -> Result<Sha256Id, ArtifactError> {
        content_id(self.to_json()?.as_bytes())
    }
}

impl ReadingArtifact {
    pub const fn new(payload: TarotReading) -> Self {
        Self { payload }
    }

    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        match Artifact::from_json(input)? {
            Artifact::Reading(artifact) => Ok(*artifact),
            Artifact::Deck(_) => Err(ArtifactError::UnexpectedType {
                expected: ArtifactKind::Reading,
                actual: ArtifactKind::Deck,
            }),
        }
    }

    pub const fn payload(&self) -> &TarotReading {
        &self.payload
    }

    pub fn into_payload(self) -> TarotReading {
        self.payload
    }

    pub fn to_json(&self) -> Result<String, ArtifactError> {
        serialize_compact(ArtifactKind::Reading, &self.payload)
    }

    pub fn to_pretty_json(&self) -> Result<String, ArtifactError> {
        serialize_pretty(ArtifactKind::Reading, &self.payload)
    }

    pub fn content_id(&self) -> Result<Sha256Id, ArtifactError> {
        content_id(self.to_json()?.as_bytes())
    }
}

impl Artifact {
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let wire: EnvelopeWire = serde_json::from_str(input)?;
        match wire {
            EnvelopeWire::Deck {
                schema_version,
                payload,
            } => {
                ensure_supported(schema_version)?;
                Ok(Self::Deck(Box::new(DeckArtifact::new(*payload))))
            }
            EnvelopeWire::Reading {
                schema_version,
                payload,
            } => {
                ensure_supported(schema_version)?;
                Ok(Self::Reading(Box::new(ReadingArtifact::new(*payload))))
            }
        }
    }

    pub const fn kind(&self) -> ArtifactKind {
        match self {
            Self::Deck(_) => ArtifactKind::Deck,
            Self::Reading(_) => ArtifactKind::Reading,
        }
    }

    pub fn to_json(&self) -> Result<String, ArtifactError> {
        match self {
            Self::Deck(artifact) => artifact.to_json(),
            Self::Reading(artifact) => artifact.to_json(),
        }
    }

    pub fn to_pretty_json(&self) -> Result<String, ArtifactError> {
        match self {
            Self::Deck(artifact) => artifact.to_pretty_json(),
            Self::Reading(artifact) => artifact.to_pretty_json(),
        }
    }

    pub fn content_id(&self) -> Result<Sha256Id, ArtifactError> {
        match self {
            Self::Deck(artifact) => artifact.content_id(),
            Self::Reading(artifact) => artifact.content_id(),
        }
    }
}

fn ensure_supported(schema_version: u32) -> Result<(), ArtifactError> {
    if schema_version == ARTIFACT_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ArtifactError::UnsupportedSchema(schema_version))
    }
}

fn serialize_compact<T: Serialize>(
    artifact_type: ArtifactKind,
    payload: &T,
) -> Result<String, ArtifactError> {
    Ok(serde_json::to_string(&EnvelopeRef {
        schema_version: ARTIFACT_SCHEMA_VERSION,
        artifact_type,
        payload,
    })?)
}

fn serialize_pretty<T: Serialize>(
    artifact_type: ArtifactKind,
    payload: &T,
) -> Result<String, ArtifactError> {
    Ok(serde_json::to_string_pretty(&EnvelopeRef {
        schema_version: ARTIFACT_SCHEMA_VERSION,
        artifact_type,
        payload,
    })?)
}

fn content_id(canonical_bytes: &[u8]) -> Result<Sha256Id, ArtifactError> {
    let digest: [u8; 32] = Sha256::digest(canonical_bytes).into();
    Ok(Sha256Id::from_digest(digest))
}
