use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::{
    StableId, ValidationError,
    validation::{validate_optional_text, validate_text},
};

pub const SPREAD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpreadLayout {
    Fixed,
    Freeform,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct LayoutHint {
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutHintWire {
    x: f64,
    y: f64,
}

impl LayoutHint {
    pub fn new(x: f64, y: f64) -> Result<Self, ValidationError> {
        if !x.is_finite() {
            return Err(ValidationError::NonFinite { field: "layout.x" });
        }
        if !y.is_finite() {
            return Err(ValidationError::NonFinite { field: "layout.y" });
        }
        Ok(Self { x, y })
    }

    pub const fn x(self) -> f64 {
        self.x
    }
    pub const fn y(self) -> f64 {
        self.y
    }
}

impl<'de> Deserialize<'de> for LayoutHint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = LayoutHintWire::deserialize(deserializer)?;
        Self::new(wire.x, wire.y).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SpreadPosition {
    id: StableId,
    label: String,
    meaning: Option<String>,
    layout_hint: Option<LayoutHint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpreadPositionWire {
    id: StableId,
    label: String,
    meaning: Option<String>,
    layout_hint: Option<LayoutHint>,
}

impl SpreadPosition {
    pub fn new(
        id: StableId,
        label: impl Into<String>,
        meaning: Option<String>,
        layout_hint: Option<LayoutHint>,
    ) -> Result<Self, ValidationError> {
        let label = label.into();
        validate_text("spread_position.label", &label)?;
        validate_optional_text("spread_position.meaning", meaning.as_deref())?;
        Ok(Self {
            id,
            label,
            meaning,
            layout_hint,
        })
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn meaning(&self) -> Option<&str> {
        self.meaning.as_deref()
    }
    pub const fn layout_hint(&self) -> Option<LayoutHint> {
        self.layout_hint
    }
}

impl<'de> Deserialize<'de> for SpreadPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SpreadPositionWire::deserialize(deserializer)?;
        Self::new(wire.id, wire.label, wire.meaning, wire.layout_hint)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpreadDefinition {
    id: StableId,
    name: String,
    layout: SpreadLayout,
    positions: Vec<SpreadPosition>,
}

#[derive(Serialize)]
struct SpreadDefinitionRef<'a> {
    schema_version: u32,
    id: &'a StableId,
    name: &'a str,
    layout: SpreadLayout,
    positions: &'a [SpreadPosition],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpreadDefinitionWire {
    schema_version: u32,
    id: StableId,
    name: String,
    layout: SpreadLayout,
    positions: Vec<SpreadPosition>,
}

#[derive(Debug, Error)]
pub enum SpreadError {
    #[error("invalid spread JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported spread schema version {0}")]
    UnsupportedSchema(u32),
    #[error(transparent)]
    InvalidSpread(#[from] ValidationError),
}

impl SpreadDefinition {
    pub fn new(
        id: StableId,
        name: impl Into<String>,
        layout: SpreadLayout,
        positions: Vec<SpreadPosition>,
    ) -> Result<Self, ValidationError> {
        let name = name.into();
        validate_text("spread.name", &name)?;
        if positions.is_empty() {
            return Err(ValidationError::EmptySpread);
        }
        let mut ids = BTreeSet::new();
        for position in &positions {
            if !ids.insert(position.id().clone()) {
                return Err(ValidationError::DuplicateSpreadPositionId(
                    position.id().clone(),
                ));
            }
        }
        Ok(Self {
            id,
            name,
            layout,
            positions,
        })
    }

    pub fn one_card() -> Self {
        Self::new(
            StableId::new("spread_id", "one_card").expect("valid built-in spread ID"),
            "One Card",
            SpreadLayout::Fixed,
            vec![
                SpreadPosition::new(
                    StableId::new("position_id", "card").expect("valid built-in position ID"),
                    "Card",
                    None,
                    Some(LayoutHint::new(0.0, 0.0).expect("finite built-in layout")),
                )
                .expect("valid built-in position"),
            ],
        )
        .expect("valid built-in spread")
    }

    pub fn three_card(
        id: StableId,
        name: impl Into<String>,
        positions: [SpreadPosition; 3],
    ) -> Result<Self, ValidationError> {
        Self::new(id, name, SpreadLayout::Fixed, positions.into())
    }

    pub fn from_json(input: &str) -> Result<Self, SpreadError> {
        let wire: SpreadDefinitionWire = serde_json::from_str(input)?;
        if wire.schema_version != SPREAD_SCHEMA_VERSION {
            return Err(SpreadError::UnsupportedSchema(wire.schema_version));
        }
        Ok(Self::new(wire.id, wire.name, wire.layout, wire.positions)?)
    }

    pub fn id(&self) -> &StableId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn layout(&self) -> SpreadLayout {
        self.layout
    }
    pub fn positions(&self) -> &[SpreadPosition] {
        &self.positions
    }
    pub fn position(&self, id: &StableId) -> Option<&SpreadPosition> {
        self.positions.iter().find(|position| position.id() == id)
    }

    pub fn to_json(&self) -> Result<String, SpreadError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_pretty_json(&self) -> Result<String, SpreadError> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

impl Serialize for SpreadDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SpreadDefinitionRef {
            schema_version: SPREAD_SCHEMA_VERSION,
            id: &self.id,
            name: &self.name,
            layout: self.layout,
            positions: &self.positions,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SpreadDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SpreadDefinitionWire::deserialize(deserializer)?;
        if wire.schema_version != SPREAD_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format_args!(
                "unsupported spread schema version {}",
                wire.schema_version
            )));
        }
        Self::new(wire.id, wire.name, wire.layout, wire.positions).map_err(serde::de::Error::custom)
    }
}
