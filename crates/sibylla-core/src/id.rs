use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::ValidationError;

/// A stable, display-independent identifier used by tarot domain records.
///
/// IDs begin with an ASCII lowercase letter and otherwise contain only ASCII
/// lowercase letters, digits, and underscores. Namespacing will be represented
/// explicitly by higher-level identity types rather than embedded punctuation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct StableId(String);

impl StableId {
    pub fn new(field: &'static str, value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let mut bytes = value.bytes();
        if !matches!(bytes.next(), Some(b'a'..=b'z'))
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(ValidationError::InvalidStableId { field });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StableId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for StableId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new("stable_id", value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_card_style_ids() {
        let id = StableId::new("card_id", "seven_of_cups").unwrap();
        assert_eq!(id.as_str(), "seven_of_cups");
    }

    #[test]
    fn rejects_display_text_and_empty_ids() {
        for value in ["", "Seven of Cups", "seven-of-cups", "7_cups"] {
            assert!(StableId::new("card_id", value).is_err(), "{value}");
        }
    }

    #[test]
    fn deserialization_cannot_bypass_validation() {
        assert!(serde_json::from_str::<StableId>(r#""Queen of Swords""#).is_err());
        assert_eq!(
            serde_json::from_str::<StableId>(r#""queen_of_swords""#)
                .unwrap()
                .as_str(),
            "queen_of_swords"
        );
    }
}
