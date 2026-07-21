//! Versioned, unbiased tarot shuffling with operating-system entropy by default.

use std::num::NonZeroU32;

use sha2::{Digest, Sha256};
use sibylla_core::{
    CardIdentity, DeckManifest, DrawProvenance, Orientation, RandomnessSource, ReversalPolicy,
    Sha256Id, StableId, UtcInstant, ValidationError,
};
use thiserror::Error;

pub const ALGORITHM_ID: &str = "fisher_yates";
pub const ALGORITHM_VERSION: u32 = 1;

/// An entropy failure with no partial shuffle result.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{0}")]
pub struct RandomSourceError(String);

impl RandomSourceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// An injected entropy boundary for deterministic contract tests and replays.
///
/// Production callers should use [`shuffle`], which always selects [`OsRandom`].
pub trait RandomSource {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), RandomSourceError>;
}

/// Operating-system cryptographic randomness used by the production entrypoint.
#[derive(Clone, Copy, Debug, Default)]
pub struct OsRandom;

impl RandomSource for OsRandom {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), RandomSourceError> {
        getrandom::fill(destination).map_err(|error| {
            RandomSourceError::new(format!("operating-system entropy failed: {error}"))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShuffledCard {
    deck_card_id: StableId,
    card_identity: CardIdentity,
    printed_title: String,
    orientation: Orientation,
}

impl ShuffledCard {
    pub fn deck_card_id(&self) -> &StableId {
        &self.deck_card_id
    }
    pub fn card_identity(&self) -> &CardIdentity {
        &self.card_identity
    }
    pub fn printed_title(&self) -> &str {
        &self.printed_title
    }
    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShuffledDeck {
    cards: Vec<ShuffledCard>,
    provenance: DrawProvenance,
}

impl ShuffledDeck {
    pub fn cards(&self) -> &[ShuffledCard] {
        &self.cards
    }
    pub fn provenance(&self) -> &DrawProvenance {
        &self.provenance
    }
}

#[derive(Debug, Error)]
pub enum ShuffleError {
    #[error(transparent)]
    Entropy(#[from] RandomSourceError),
    #[error("enabled deck population is too large: {0}")]
    DeckTooLarge(usize),
    #[error("manifest hashing failed: {0}")]
    Manifest(#[from] sibylla_core::ManifestError),
    #[error("shuffle provenance is invalid: {0}")]
    InvalidProvenance(#[from] ValidationError),
}

/// Securely shuffle a manifest's exact enabled population using OS randomness.
pub fn shuffle(
    manifest: &DeckManifest,
    shuffled_at: UtcInstant,
) -> Result<ShuffledDeck, ShuffleError> {
    shuffle_inner(
        manifest,
        shuffled_at,
        &mut OsRandom,
        RandomnessSource::OperatingSystem,
        None,
    )
}

/// Shuffle with an injected source for deterministic tests or explicit replays.
///
/// This is never called by the production [`shuffle`] entrypoint.
pub fn shuffle_with_source<R: RandomSource>(
    manifest: &DeckManifest,
    shuffled_at: UtcInstant,
    source: &mut R,
    seed_commitment: Option<Sha256Id>,
) -> Result<ShuffledDeck, ShuffleError> {
    shuffle_inner(
        manifest,
        shuffled_at,
        source,
        RandomnessSource::Injected,
        seed_commitment,
    )
}

fn shuffle_inner<R: RandomSource>(
    manifest: &DeckManifest,
    shuffled_at: UtcInstant,
    source: &mut R,
    randomness_source: RandomnessSource,
    seed_commitment: Option<Sha256Id>,
) -> Result<ShuffledDeck, ShuffleError> {
    let mut cards: Vec<_> = manifest
        .enabled_cards()
        .map(|card| ShuffledCard {
            deck_card_id: card.id().clone(),
            card_identity: card.identity().clone(),
            printed_title: card.printed_title().to_owned(),
            orientation: Orientation::Upright,
        })
        .collect();

    for upper in (2..=cards.len()).rev() {
        let selected = uniform_below(source, upper)?;
        cards.swap(upper - 1, selected);
    }

    let reversal_policy = manifest.reversal_policy();
    for card in &mut cards {
        card.orientation = sample_orientation(source, reversal_policy)?;
    }

    let count = u32::try_from(cards.len()).map_err(|_| ShuffleError::DeckTooLarge(cards.len()))?;
    let enabled_card_count =
        NonZeroU32::new(count).expect("validated manifests always have at least one enabled card");
    let provenance = DrawProvenance::SoftwareShuffle {
        algorithm: StableId::new("shuffle.algorithm", ALGORITHM_ID)
            .expect("versioned algorithm ID is valid"),
        algorithm_version: NonZeroU32::new(ALGORITHM_VERSION)
            .expect("algorithm version is nonzero"),
        randomness_source,
        draw_manifest_id: Sha256Id::parse("draw_manifest_id", manifest.draw_manifest_id()?)?,
        enabled_card_count,
        reversal_policy,
        shuffled_at,
        seed_commitment,
    };
    Ok(ShuffledDeck { cards, provenance })
}

/// Commit to seed bytes without retaining or exposing the seed itself.
pub fn seed_commitment(seed: &[u8]) -> Sha256Id {
    Sha256Id::from_digest(Sha256::digest(seed).into())
}

fn sample_orientation<R: RandomSource>(
    source: &mut R,
    policy: ReversalPolicy,
) -> Result<Orientation, RandomSourceError> {
    match policy.rate_basis_points() {
        0 => Ok(Orientation::Upright),
        10_000 => Ok(Orientation::Reversed),
        rate => Ok(if uniform_below(source, 10_000)? < usize::from(rate) {
            Orientation::Reversed
        } else {
            Orientation::Upright
        }),
    }
}

fn uniform_below<R: RandomSource>(
    source: &mut R,
    upper: usize,
) -> Result<usize, RandomSourceError> {
    debug_assert!(upper > 0);
    let upper = upper as u128;
    let zone = (u128::from(u64::MAX) + 1) / upper * upper;
    loop {
        let mut bytes = [0_u8; 8];
        source.fill_bytes(&mut bytes)?;
        let sample = u128::from(u64::from_le_bytes(bytes));
        if sample < zone {
            return Ok((sample % upper) as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SequenceSource {
        values: Vec<u64>,
        calls: usize,
    }

    impl RandomSource for SequenceSource {
        fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), RandomSourceError> {
            let value = self.values[self.calls];
            self.calls += 1;
            destination.copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
    }

    #[test]
    fn rejection_sampling_discards_the_biased_tail() {
        let mut source = SequenceSource {
            values: vec![u64::MAX, 1],
            calls: 0,
        };
        assert_eq!(uniform_below(&mut source, 3).unwrap(), 1);
        assert_eq!(source.calls, 2);
    }
}
