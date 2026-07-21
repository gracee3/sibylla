use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{StableId, ValidationError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arcana {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MajorArcana {
    Chariot,
    Death,
    Devil,
    Emperor,
    Empress,
    Fool,
    HangedMan,
    Hermit,
    Hierophant,
    HighPriestess,
    Judgement,
    Justice,
    Lovers,
    Magician,
    Moon,
    Star,
    Strength,
    Sun,
    Temperance,
    Tower,
    WheelOfFortune,
    World,
}

impl MajorArcana {
    /// All semantic major identities in stable alphabetical order.
    pub const ALL: [Self; 22] = [
        Self::Chariot,
        Self::Death,
        Self::Devil,
        Self::Emperor,
        Self::Empress,
        Self::Fool,
        Self::HangedMan,
        Self::Hermit,
        Self::Hierophant,
        Self::HighPriestess,
        Self::Judgement,
        Self::Justice,
        Self::Lovers,
        Self::Magician,
        Self::Moon,
        Self::Star,
        Self::Strength,
        Self::Sun,
        Self::Temperance,
        Self::Tower,
        Self::WheelOfFortune,
        Self::World,
    ];

    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Chariot => "chariot",
            Self::Death => "death",
            Self::Devil => "devil",
            Self::Emperor => "emperor",
            Self::Empress => "empress",
            Self::Fool => "fool",
            Self::HangedMan => "hanged_man",
            Self::Hermit => "hermit",
            Self::Hierophant => "hierophant",
            Self::HighPriestess => "high_priestess",
            Self::Judgement => "judgement",
            Self::Justice => "justice",
            Self::Lovers => "lovers",
            Self::Magician => "magician",
            Self::Moon => "moon",
            Self::Star => "star",
            Self::Strength => "strength",
            Self::Sun => "sun",
            Self::Temperance => "temperance",
            Self::Tower => "tower",
            Self::WheelOfFortune => "wheel_of_fortune",
            Self::World => "world",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinorSuit {
    Wands,
    Cups,
    Swords,
    Pentacles,
}

impl MinorSuit {
    pub const ALL: [Self; 4] = [Self::Wands, Self::Cups, Self::Swords, Self::Pentacles];

    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Wands => "wands",
            Self::Cups => "cups",
            Self::Swords => "swords",
            Self::Pentacles => "pentacles",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinorRank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Page,
    Knight,
    Queen,
    King,
}

impl MinorRank {
    pub const ALL: [Self; 14] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Page,
        Self::Knight,
        Self::Queen,
        Self::King,
    ];

    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Ace => "ace",
            Self::Two => "two",
            Self::Three => "three",
            Self::Four => "four",
            Self::Five => "five",
            Self::Six => "six",
            Self::Seven => "seven",
            Self::Eight => "eight",
            Self::Nine => "nine",
            Self::Ten => "ten",
            Self::Page => "page",
            Self::Knight => "knight",
            Self::Queen => "queen",
            Self::King => "king",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConventionalCard {
    Major(MajorArcana),
    Minor { suit: MinorSuit, rank: MinorRank },
}

impl ConventionalCard {
    pub const fn arcana(self) -> Arcana {
        match self {
            Self::Major(_) => Arcana::Major,
            Self::Minor { .. } => Arcana::Minor,
        }
    }

    pub fn stable_id(self) -> String {
        match self {
            Self::Major(major) => major.stable_id().to_owned(),
            Self::Minor { suit, rank } => {
                format!("{}_of_{}", rank.stable_id(), suit.stable_id())
            }
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        MajorArcana::ALL
            .into_iter()
            .map(Self::Major)
            .chain(MinorSuit::ALL.into_iter().flat_map(|suit| {
                MinorRank::ALL
                    .into_iter()
                    .map(move |rank| Self::Minor { suit, rank })
            }))
    }
}

impl FromStr for ConventionalCard {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::all()
            .find(|card| card.stable_id() == value)
            .ok_or_else(|| ValidationError::UnknownConventionalCard(value.to_owned()))
    }
}

impl fmt::Display for ConventionalCard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.stable_id())
    }
}

impl Serialize for ConventionalCard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.stable_id().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConventionalCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CardIdentity {
    Conventional(ConventionalCard),
    Extension { namespace: StableId, id: StableId },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CardIdentityRef<'a> {
    Conventional {
        id: &'a ConventionalCard,
    },
    Extension {
        namespace: &'a StableId,
        id: &'a StableId,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum CardIdentityWire {
    Conventional { id: ConventionalCard },
    Extension { namespace: StableId, id: StableId },
}

impl Serialize for CardIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Conventional(id) => CardIdentityRef::Conventional { id }.serialize(serializer),
            Self::Extension { namespace, id } => {
                CardIdentityRef::Extension { namespace, id }.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for CardIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match CardIdentityWire::deserialize(deserializer)? {
            CardIdentityWire::Conventional { id } => Self::Conventional(id),
            CardIdentityWire::Extension { namespace, id } => Self::Extension { namespace, id },
        })
    }
}
