use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{CardIdentity, StableId, ValidationError};

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const DRAW_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Attribution {
    author: Option<String>,
    artist: Option<String>,
    publisher: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttributionWire {
    author: Option<String>,
    artist: Option<String>,
    publisher: Option<String>,
}

impl Attribution {
    pub fn new(
        author: Option<String>,
        artist: Option<String>,
        publisher: Option<String>,
    ) -> Result<Self, ValidationError> {
        validate_optional_text("author", author.as_deref())?;
        validate_optional_text("artist", artist.as_deref())?;
        validate_optional_text("publisher", publisher.as_deref())?;
        Ok(Self {
            author,
            artist,
            publisher,
        })
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }
    pub fn publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }
}

impl<'de> Deserialize<'de> for Attribution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AttributionWire::deserialize(deserializer)?;
        Self::new(wire.author, wire.artist, wire.publisher).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct RightsMetadata {
    license: Option<String>,
    source: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RightsMetadataWire {
    license: Option<String>,
    source: Option<String>,
    notes: Option<String>,
}

impl RightsMetadata {
    pub fn new(
        license: Option<String>,
        source: Option<String>,
        notes: Option<String>,
    ) -> Result<Self, ValidationError> {
        validate_optional_text("rights.license", license.as_deref())?;
        validate_optional_text("rights.source", source.as_deref())?;
        validate_optional_text("rights.notes", notes.as_deref())?;
        Ok(Self {
            license,
            source,
            notes,
        })
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

impl<'de> Deserialize<'de> for RightsMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RightsMetadataWire::deserialize(deserializer)?;
        Self::new(wire.license, wire.source, wire.notes).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ReversalPolicy(u16);

impl ReversalPolicy {
    pub fn new(rate_basis_points: u16) -> Result<Self, ValidationError> {
        if rate_basis_points > 10_000 {
            return Err(ValidationError::InvalidReversalRate(rate_basis_points));
        }
        Ok(Self(rate_basis_points))
    }

    pub const fn rate_basis_points(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ReversalPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Correspondence {
    key: StableId,
    value: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorrespondenceWire {
    key: StableId,
    value: String,
}

impl Correspondence {
    pub fn new(key: StableId, value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        validate_text("correspondence.value", &value)?;
        Ok(Self { key, value })
    }

    pub fn key(&self) -> &StableId {
        &self.key
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl<'de> Deserialize<'de> for Correspondence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CorrespondenceWire::deserialize(deserializer)?;
        Self::new(wire.key, wire.value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeckCard {
    id: StableId,
    identity: CardIdentity,
    printed_title: String,
    printed_number: Option<String>,
    printed_suit: Option<String>,
    printed_rank: Option<String>,
    enabled: bool,
    asset_id: Option<StableId>,
    correspondences: Vec<Correspondence>,
    notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeckCardWire {
    id: StableId,
    identity: CardIdentity,
    printed_title: String,
    printed_number: Option<String>,
    printed_suit: Option<String>,
    printed_rank: Option<String>,
    enabled: bool,
    asset_id: Option<StableId>,
    correspondences: Vec<Correspondence>,
    notes: Option<String>,
}

impl DeckCard {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: StableId,
        identity: CardIdentity,
        printed_title: impl Into<String>,
        printed_number: Option<String>,
        printed_suit: Option<String>,
        printed_rank: Option<String>,
        enabled: bool,
        asset_id: Option<StableId>,
        correspondences: Vec<Correspondence>,
        notes: Option<String>,
    ) -> Result<Self, ValidationError> {
        let printed_title = printed_title.into();
        validate_text("printed_title", &printed_title)?;
        validate_optional_text("printed_number", printed_number.as_deref())?;
        validate_optional_text("printed_suit", printed_suit.as_deref())?;
        validate_optional_text("printed_rank", printed_rank.as_deref())?;
        validate_optional_text("deck_card.notes", notes.as_deref())?;
        let mut keys = BTreeSet::new();
        for correspondence in &correspondences {
            if !keys.insert(correspondence.key().clone()) {
                return Err(ValidationError::DuplicateCorrespondenceKey(
                    correspondence.key().clone(),
                ));
            }
        }
        Ok(Self {
            id,
            identity,
            printed_title,
            printed_number,
            printed_suit,
            printed_rank,
            enabled,
            asset_id,
            correspondences,
            notes,
        })
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub fn identity(&self) -> &CardIdentity {
        &self.identity
    }
    pub fn printed_title(&self) -> &str {
        &self.printed_title
    }
    pub fn printed_number(&self) -> Option<&str> {
        self.printed_number.as_deref()
    }
    pub fn printed_suit(&self) -> Option<&str> {
        self.printed_suit.as_deref()
    }
    pub fn printed_rank(&self) -> Option<&str> {
        self.printed_rank.as_deref()
    }
    pub const fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn asset_id(&self) -> Option<&StableId> {
        self.asset_id.as_ref()
    }
    pub fn correspondences(&self) -> &[Correspondence] {
        &self.correspondences
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

impl<'de> Deserialize<'de> for DeckCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DeckCardWire::deserialize(deserializer)?;
        Self::new(
            wire.id,
            wire.identity,
            wire.printed_title,
            wire.printed_number,
            wire.printed_suit,
            wire.printed_rank,
            wire.enabled,
            wire.asset_id,
            wire.correspondences,
            wire.notes,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckManifest {
    id: StableId,
    name: String,
    attribution: Attribution,
    tradition: Option<String>,
    rights: RightsMetadata,
    reversal_policy: ReversalPolicy,
    cards: Vec<DeckCard>,
}

#[derive(Serialize)]
struct DeckManifestRef<'a> {
    schema_version: u32,
    id: &'a StableId,
    name: &'a str,
    attribution: &'a Attribution,
    tradition: &'a Option<String>,
    rights: &'a RightsMetadata,
    reversal_rate_basis_points: ReversalPolicy,
    cards: &'a [DeckCard],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeckManifestWire {
    schema_version: u32,
    id: StableId,
    name: String,
    attribution: Attribution,
    tradition: Option<String>,
    rights: RightsMetadata,
    reversal_rate_basis_points: ReversalPolicy,
    cards: Vec<DeckCard>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported deck manifest schema version {0}")]
    UnsupportedSchema(u32),
    #[error(transparent)]
    InvalidManifest(#[from] ValidationError),
}

impl DeckManifest {
    pub fn new(
        id: StableId,
        name: impl Into<String>,
        attribution: Attribution,
        tradition: Option<String>,
        rights: RightsMetadata,
        reversal_policy: ReversalPolicy,
        cards: Vec<DeckCard>,
    ) -> Result<Self, ValidationError> {
        let name = name.into();
        validate_text("deck.name", &name)?;
        validate_optional_text("deck.tradition", tradition.as_deref())?;
        if cards.is_empty() {
            return Err(ValidationError::EmptyDeck);
        }
        if !cards.iter().any(DeckCard::enabled) {
            return Err(ValidationError::NoEnabledCards);
        }
        let mut ids = BTreeSet::new();
        for card in &cards {
            if !ids.insert(card.id().clone()) {
                return Err(ValidationError::DuplicateDeckCardId(card.id().clone()));
            }
        }
        Ok(Self {
            id,
            name,
            attribution,
            tradition,
            rights,
            reversal_policy,
            cards,
        })
    }

    pub fn from_json(input: &str) -> Result<Self, ManifestError> {
        let wire: DeckManifestWire = serde_json::from_str(input)?;
        if wire.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema(wire.schema_version));
        }
        Ok(Self::new(
            wire.id,
            wire.name,
            wire.attribution,
            wire.tradition,
            wire.rights,
            wire.reversal_rate_basis_points,
            wire.cards,
        )?)
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn attribution(&self) -> &Attribution {
        &self.attribution
    }
    pub fn tradition(&self) -> Option<&str> {
        self.tradition.as_deref()
    }
    pub fn rights(&self) -> &RightsMetadata {
        &self.rights
    }
    pub const fn reversal_policy(&self) -> ReversalPolicy {
        self.reversal_policy
    }
    pub fn cards(&self) -> &[DeckCard] {
        &self.cards
    }
    pub fn enabled_cards(&self) -> impl Iterator<Item = &DeckCard> {
        self.cards.iter().filter(|card| card.enabled())
    }

    pub fn to_json(&self) -> Result<String, ManifestError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_pretty_json(&self) -> Result<String, ManifestError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn content_id(&self) -> Result<String, ManifestError> {
        sha256_id(&serde_json::to_vec(self)?)
    }

    pub fn draw_manifest_id(&self) -> Result<String, ManifestError> {
        let cards: Vec<_> = self
            .enabled_cards()
            .map(|card| DrawCardRef {
                deck_card_id: card.id(),
                identity: card.identity(),
            })
            .collect();
        sha256_id(&serde_json::to_vec(&DrawManifestRef {
            schema_version: DRAW_MANIFEST_SCHEMA_VERSION,
            manifest_id: &self.id,
            reversal_rate_basis_points: self.reversal_policy,
            cards,
        })?)
    }
}

impl Serialize for DeckManifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DeckManifestRef {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: &self.id,
            name: &self.name,
            attribution: &self.attribution,
            tradition: &self.tradition,
            rights: &self.rights,
            reversal_rate_basis_points: self.reversal_policy,
            cards: &self.cards,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DeckManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DeckManifestWire::deserialize(deserializer)?;
        if wire.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format_args!(
                "unsupported deck manifest schema version {}",
                wire.schema_version
            )));
        }
        Self::new(
            wire.id,
            wire.name,
            wire.attribution,
            wire.tradition,
            wire.rights,
            wire.reversal_rate_basis_points,
            wire.cards,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize)]
struct DrawManifestRef<'a> {
    schema_version: u32,
    manifest_id: &'a StableId,
    reversal_rate_basis_points: ReversalPolicy,
    cards: Vec<DrawCardRef<'a>>,
}

#[derive(Serialize)]
struct DrawCardRef<'a> {
    deck_card_id: &'a StableId,
    identity: &'a CardIdentity,
}

fn validate_text(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::EmptyText { field })
    } else {
        Ok(())
    }
}

fn validate_optional_text(field: &'static str, value: Option<&str>) -> Result<(), ValidationError> {
    if let Some(value) = value {
        validate_text(field, value)?;
    }
    Ok(())
}

fn sha256_id(bytes: &[u8]) -> Result<String, ManifestError> {
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}
