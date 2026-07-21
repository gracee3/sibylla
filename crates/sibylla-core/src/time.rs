use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ValidationError;

/// A caller-supplied timestamp normalized to UTC.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UtcInstant(DateTime<Utc>);

impl UtcInstant {
    pub fn parse_rfc3339(value: &str) -> Result<Self, ValidationError> {
        DateTime::parse_from_rfc3339(value)
            .map(|instant| Self(instant.with_timezone(&Utc)))
            .map_err(|_| ValidationError::InvalidUtcInstant(value.to_owned()))
    }

    pub const fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn to_rfc3339(self) -> String {
        self.0.to_rfc3339_opts(SecondsFormat::AutoSi, true)
    }
}

impl Serialize for UtcInstant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_rfc3339().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UtcInstant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse_rfc3339(&value).map_err(serde::de::Error::custom)
    }
}
