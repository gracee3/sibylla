use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::ValidationError;

/// A validated lowercase SHA-256 content identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Sha256Id(String);

impl Sha256Id {
    pub fn parse(field: &'static str, value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let Some(digest) = value.strip_prefix("sha256:") else {
            return Err(ValidationError::InvalidSha256Id { field });
        };
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ValidationError::InvalidSha256Id { field });
        }
        Ok(Self(value))
    }

    pub fn from_digest(digest: [u8; 32]) -> Self {
        let mut value = String::with_capacity(71);
        value.push_str("sha256:");
        for byte in digest {
            use std::fmt::Write;
            write!(value, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Sha256Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Sha256Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse("sha256_id", value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_formats_sha256_ids() {
        let id = Sha256Id::from_digest([0xab; 32]);
        assert_eq!(id.as_str(), format!("sha256:{}", "ab".repeat(32)));
        assert_eq!(Sha256Id::parse("content_id", id.to_string()).unwrap(), id);
        assert!(Sha256Id::parse("content_id", "sha256:ABC").is_err());
    }
}
