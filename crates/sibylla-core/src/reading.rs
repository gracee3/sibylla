use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::{
    CardIdentity, DeckManifest, SpreadDefinition, SpreadLayout, StableId, UtcInstant,
    ValidationError,
    validation::{validate_optional_text, validate_text},
};

pub const READING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Upright,
    Reversed,
    Unspecified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum DrawProvenance {
    Manual { recorded_at: UtcInstant },
}

impl DrawProvenance {
    pub const fn recorded_at(self) -> UtcInstant {
        match self {
            Self::Manual { recorded_at } => recorded_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Placement {
    position_id: StableId,
    position_label: String,
    card_identity: CardIdentity,
    deck_card_id: StableId,
    printed_title: String,
    orientation: Orientation,
    draw_order: u32,
    notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementWire {
    position_id: StableId,
    position_label: String,
    card_identity: CardIdentity,
    deck_card_id: StableId,
    printed_title: String,
    orientation: Orientation,
    draw_order: u32,
    notes: Option<String>,
}

impl Placement {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position_id: StableId,
        position_label: impl Into<String>,
        card_identity: CardIdentity,
        deck_card_id: StableId,
        printed_title: impl Into<String>,
        orientation: Orientation,
        draw_order: u32,
        notes: Option<String>,
    ) -> Result<Self, ValidationError> {
        let position_label = position_label.into();
        let printed_title = printed_title.into();
        validate_text("placement.position_label", &position_label)?;
        validate_text("placement.printed_title", &printed_title)?;
        validate_optional_text("placement.notes", notes.as_deref())?;
        Ok(Self {
            position_id,
            position_label,
            card_identity,
            deck_card_id,
            printed_title,
            orientation,
            draw_order,
            notes,
        })
    }

    pub fn position_id(&self) -> &StableId {
        &self.position_id
    }
    pub fn position_label(&self) -> &str {
        &self.position_label
    }
    pub fn card_identity(&self) -> &CardIdentity {
        &self.card_identity
    }
    pub fn deck_card_id(&self) -> &StableId {
        &self.deck_card_id
    }
    pub fn printed_title(&self) -> &str {
        &self.printed_title
    }
    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }
    pub const fn draw_order(&self) -> u32 {
        self.draw_order
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

impl<'de> Deserialize<'de> for Placement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PlacementWire::deserialize(deserializer)?;
        Self::new(
            wire.position_id,
            wire.position_label,
            wire.card_identity,
            wire.deck_card_id,
            wire.printed_title,
            wire.orientation,
            wire.draw_order,
            wire.notes,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowUpKind {
    Annotation,
    Outcome,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FollowUp {
    id: StableId,
    kind: FollowUpKind,
    content: String,
    created_at: UtcInstant,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FollowUpWire {
    id: StableId,
    kind: FollowUpKind,
    content: String,
    created_at: UtcInstant,
}

impl FollowUp {
    pub fn new(
        id: StableId,
        kind: FollowUpKind,
        content: impl Into<String>,
        created_at: UtcInstant,
    ) -> Result<Self, ValidationError> {
        let content = content.into();
        validate_text("follow_up.content", &content)?;
        Ok(Self {
            id,
            kind,
            content,
            created_at,
        })
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub const fn kind(&self) -> FollowUpKind {
        self.kind
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub const fn created_at(&self) -> UtcInstant {
        self.created_at
    }
}

impl<'de> Deserialize<'de> for FollowUp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = FollowUpWire::deserialize(deserializer)?;
        Self::new(wire.id, wire.kind, wire.content, wire.created_at)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TarotReading {
    id: StableId,
    subject_ref: Option<String>,
    session_ref: Option<String>,
    deck: DeckManifest,
    spread: SpreadDefinition,
    question: Option<String>,
    context: Option<String>,
    placements: Vec<Placement>,
    draw_provenance: DrawProvenance,
    reader_notes: Option<String>,
    interpretation: Option<String>,
    follow_ups: Vec<FollowUp>,
    created_at: UtcInstant,
    modified_at: UtcInstant,
}

#[derive(Serialize)]
struct TarotReadingRef<'a> {
    schema_version: u32,
    id: &'a StableId,
    subject_ref: &'a Option<String>,
    session_ref: &'a Option<String>,
    deck: &'a DeckManifest,
    spread: &'a SpreadDefinition,
    question: &'a Option<String>,
    context: &'a Option<String>,
    placements: &'a [Placement],
    draw_provenance: DrawProvenance,
    reader_notes: &'a Option<String>,
    interpretation: &'a Option<String>,
    follow_ups: &'a [FollowUp],
    created_at: UtcInstant,
    modified_at: UtcInstant,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TarotReadingWire {
    schema_version: u32,
    id: StableId,
    subject_ref: Option<String>,
    session_ref: Option<String>,
    deck: DeckManifest,
    spread: SpreadDefinition,
    question: Option<String>,
    context: Option<String>,
    placements: Vec<Placement>,
    draw_provenance: DrawProvenance,
    reader_notes: Option<String>,
    interpretation: Option<String>,
    follow_ups: Vec<FollowUp>,
    created_at: UtcInstant,
    modified_at: UtcInstant,
}

#[derive(Debug, Error)]
pub enum ReadingError {
    #[error("invalid reading JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported reading schema version {0}")]
    UnsupportedSchema(u32),
    #[error(transparent)]
    InvalidReading(#[from] ValidationError),
}

impl TarotReading {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: StableId,
        subject_ref: Option<String>,
        session_ref: Option<String>,
        deck: DeckManifest,
        spread: SpreadDefinition,
        question: Option<String>,
        context: Option<String>,
        placements: Vec<Placement>,
        draw_provenance: DrawProvenance,
        reader_notes: Option<String>,
        interpretation: Option<String>,
        follow_ups: Vec<FollowUp>,
        created_at: UtcInstant,
        modified_at: UtcInstant,
    ) -> Result<Self, ValidationError> {
        validate_optional_text("reading.subject_ref", subject_ref.as_deref())?;
        validate_optional_text("reading.session_ref", session_ref.as_deref())?;
        validate_optional_text("reading.question", question.as_deref())?;
        validate_optional_text("reading.context", context.as_deref())?;
        validate_optional_text("reading.reader_notes", reader_notes.as_deref())?;
        validate_optional_text("reading.interpretation", interpretation.as_deref())?;
        validate_timeline(created_at, modified_at, draw_provenance, &follow_ups)?;
        validate_follow_ups(&follow_ups)?;
        validate_placements(&deck, &spread, &placements)?;
        Ok(Self {
            id,
            subject_ref,
            session_ref,
            deck,
            spread,
            question,
            context,
            placements,
            draw_provenance,
            reader_notes,
            interpretation,
            follow_ups,
            created_at,
            modified_at,
        })
    }

    pub fn from_json(input: &str) -> Result<Self, ReadingError> {
        let wire: TarotReadingWire = serde_json::from_str(input)?;
        if wire.schema_version != READING_SCHEMA_VERSION {
            return Err(ReadingError::UnsupportedSchema(wire.schema_version));
        }
        Ok(wire.into_reading()?)
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub fn subject_ref(&self) -> Option<&str> {
        self.subject_ref.as_deref()
    }
    pub fn session_ref(&self) -> Option<&str> {
        self.session_ref.as_deref()
    }
    pub fn deck(&self) -> &DeckManifest {
        &self.deck
    }
    pub fn spread(&self) -> &SpreadDefinition {
        &self.spread
    }
    pub fn question(&self) -> Option<&str> {
        self.question.as_deref()
    }
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }
    pub const fn draw_provenance(&self) -> DrawProvenance {
        self.draw_provenance
    }
    pub fn reader_notes(&self) -> Option<&str> {
        self.reader_notes.as_deref()
    }
    pub fn interpretation(&self) -> Option<&str> {
        self.interpretation.as_deref()
    }
    pub fn follow_ups(&self) -> &[FollowUp] {
        &self.follow_ups
    }
    pub const fn created_at(&self) -> UtcInstant {
        self.created_at
    }
    pub const fn modified_at(&self) -> UtcInstant {
        self.modified_at
    }

    pub fn to_json(&self) -> Result<String, ReadingError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_pretty_json(&self) -> Result<String, ReadingError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn revise(
        &mut self,
        question: Option<String>,
        context: Option<String>,
        placements: Vec<Placement>,
        reader_notes: Option<String>,
        interpretation: Option<String>,
        modified_at: UtcInstant,
    ) -> Result<(), ValidationError> {
        validate_optional_text("reading.question", question.as_deref())?;
        validate_optional_text("reading.context", context.as_deref())?;
        validate_optional_text("reading.reader_notes", reader_notes.as_deref())?;
        validate_optional_text("reading.interpretation", interpretation.as_deref())?;
        if modified_at < self.modified_at {
            return Err(ValidationError::InvalidTimestampOrder {
                field: "modified_at",
            });
        }
        validate_timeline(
            self.created_at,
            modified_at,
            self.draw_provenance,
            &self.follow_ups,
        )?;
        validate_placements(&self.deck, &self.spread, &placements)?;
        self.question = question;
        self.context = context;
        self.placements = placements;
        self.reader_notes = reader_notes;
        self.interpretation = interpretation;
        self.modified_at = modified_at;
        Ok(())
    }

    pub fn append_follow_up(
        &mut self,
        follow_up: FollowUp,
        modified_at: UtcInstant,
    ) -> Result<(), ValidationError> {
        if modified_at < self.modified_at || follow_up.created_at() > modified_at {
            return Err(ValidationError::InvalidTimestampOrder {
                field: "modified_at",
            });
        }
        if follow_up.created_at() < self.created_at {
            return Err(ValidationError::InvalidTimestampOrder {
                field: "follow_up.created_at",
            });
        }
        if self
            .follow_ups
            .iter()
            .any(|existing| existing.id() == follow_up.id())
        {
            return Err(ValidationError::DuplicateFollowUpId(follow_up.id().clone()));
        }
        if self
            .follow_ups
            .last()
            .is_some_and(|last| last.created_at() > follow_up.created_at())
        {
            return Err(ValidationError::FollowUpOrder);
        }
        self.follow_ups.push(follow_up);
        self.modified_at = modified_at;
        Ok(())
    }
}

impl Serialize for TarotReading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TarotReadingRef {
            schema_version: READING_SCHEMA_VERSION,
            id: &self.id,
            subject_ref: &self.subject_ref,
            session_ref: &self.session_ref,
            deck: &self.deck,
            spread: &self.spread,
            question: &self.question,
            context: &self.context,
            placements: &self.placements,
            draw_provenance: self.draw_provenance,
            reader_notes: &self.reader_notes,
            interpretation: &self.interpretation,
            follow_ups: &self.follow_ups,
            created_at: self.created_at,
            modified_at: self.modified_at,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TarotReading {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TarotReadingWire::deserialize(deserializer)?;
        if wire.schema_version != READING_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format_args!(
                "unsupported reading schema version {}",
                wire.schema_version
            )));
        }
        wire.into_reading().map_err(serde::de::Error::custom)
    }
}

impl TarotReadingWire {
    fn into_reading(self) -> Result<TarotReading, ValidationError> {
        TarotReading::new(
            self.id,
            self.subject_ref,
            self.session_ref,
            self.deck,
            self.spread,
            self.question,
            self.context,
            self.placements,
            self.draw_provenance,
            self.reader_notes,
            self.interpretation,
            self.follow_ups,
            self.created_at,
            self.modified_at,
        )
    }
}

fn validate_placements(
    deck: &DeckManifest,
    spread: &SpreadDefinition,
    placements: &[Placement],
) -> Result<(), ValidationError> {
    if placements.is_empty() {
        return Err(ValidationError::EmptyPlacementSet);
    }
    if spread.layout() == SpreadLayout::Fixed && placements.len() != spread.positions().len() {
        return Err(ValidationError::IncompleteFixedSpread);
    }
    let mut position_ids = BTreeSet::new();
    let mut deck_card_ids = BTreeSet::new();
    let mut draw_orders = BTreeSet::new();
    for placement in placements {
        if !position_ids.insert(placement.position_id().clone()) {
            return Err(ValidationError::DuplicatePlacementPositionId(
                placement.position_id().clone(),
            ));
        }
        if !deck_card_ids.insert(placement.deck_card_id().clone()) {
            return Err(ValidationError::DuplicatePlacedDeckCard(
                placement.deck_card_id().clone(),
            ));
        }
        draw_orders.insert(placement.draw_order());
        let position = spread.position(placement.position_id()).ok_or_else(|| {
            ValidationError::UnknownSpreadPosition(placement.position_id().clone())
        })?;
        if position.label() != placement.position_label() {
            return Err(ValidationError::PositionLabelMismatch(
                placement.position_id().clone(),
            ));
        }
        let deck_card = deck
            .cards()
            .iter()
            .find(|card| card.id() == placement.deck_card_id())
            .ok_or_else(|| ValidationError::UnknownDeckCard(placement.deck_card_id().clone()))?;
        if !deck_card.enabled() {
            return Err(ValidationError::DisabledDeckCard(
                placement.deck_card_id().clone(),
            ));
        }
        if deck_card.identity() != placement.card_identity() {
            return Err(ValidationError::CardIdentityMismatch(
                placement.deck_card_id().clone(),
            ));
        }
        if deck_card.printed_title() != placement.printed_title() {
            return Err(ValidationError::PrintedTitleMismatch(
                placement.deck_card_id().clone(),
            ));
        }
    }
    if draw_orders != (1..=placements.len() as u32).collect() {
        return Err(ValidationError::InvalidDrawOrder);
    }
    Ok(())
}

fn validate_follow_ups(follow_ups: &[FollowUp]) -> Result<(), ValidationError> {
    let mut ids = BTreeSet::new();
    let mut previous = None;
    for follow_up in follow_ups {
        if !ids.insert(follow_up.id().clone()) {
            return Err(ValidationError::DuplicateFollowUpId(follow_up.id().clone()));
        }
        if previous.is_some_and(|timestamp| timestamp > follow_up.created_at()) {
            return Err(ValidationError::FollowUpOrder);
        }
        previous = Some(follow_up.created_at());
    }
    Ok(())
}

fn validate_timeline(
    created_at: UtcInstant,
    modified_at: UtcInstant,
    draw_provenance: DrawProvenance,
    follow_ups: &[FollowUp],
) -> Result<(), ValidationError> {
    if modified_at < created_at {
        return Err(ValidationError::InvalidTimestampOrder {
            field: "modified_at",
        });
    }
    if draw_provenance.recorded_at() < created_at || draw_provenance.recorded_at() > modified_at {
        return Err(ValidationError::InvalidTimestampOrder {
            field: "draw_provenance.recorded_at",
        });
    }
    if follow_ups.iter().any(|follow_up| {
        follow_up.created_at() < created_at || follow_up.created_at() > modified_at
    }) {
        return Err(ValidationError::InvalidTimestampOrder {
            field: "follow_up.created_at",
        });
    }
    Ok(())
}
