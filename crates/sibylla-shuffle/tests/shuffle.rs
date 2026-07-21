use std::collections::BTreeSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use sibylla_core::{DeckManifest, DrawProvenance, Orientation, RandomnessSource, UtcInstant};
use sibylla_shuffle::{
    ALGORITHM_ID, ALGORITHM_VERSION, RandomSource, RandomSourceError, seed_commitment, shuffle,
    shuffle_with_source,
};

const DECK_FIXTURE: &str = include_str!("../../../fixtures/decks/conventional-78-v1.json");
const READING_FIXTURE: &str = include_str!("../../../fixtures/readings/manual-three-card-v1.json");
const ALGORITHM_FIXTURE: &str = include_str!("../../../fixtures/shuffle/fisher-yates-v1.json");

#[derive(Deserialize)]
struct AlgorithmFixture {
    algorithm: String,
    algorithm_version: u32,
    test_source: String,
    seed_hex: String,
    expected: Vec<ExpectedCard>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ExpectedCard {
    deck_card_id: String,
    orientation: Orientation,
}

struct SeededSource {
    seed: Vec<u8>,
    counter: u64,
}

impl SeededSource {
    fn new(seed: Vec<u8>) -> Self {
        Self { seed, counter: 0 }
    }
}

impl RandomSource for SeededSource {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), RandomSourceError> {
        assert!(destination.len() <= 32);
        let digest = Sha256::new()
            .chain_update(&self.seed)
            .chain_update(self.counter.to_le_bytes())
            .finalize();
        self.counter += 1;
        destination.copy_from_slice(&digest[..destination.len()]);
        Ok(())
    }
}

struct SequenceSource {
    values: Vec<u64>,
    index: usize,
}

impl RandomSource for SequenceSource {
    fn fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), RandomSourceError> {
        let value = self.values[self.index];
        self.index += 1;
        destination.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

struct FailingSource;

impl RandomSource for FailingSource {
    fn fill_bytes(&mut self, _destination: &mut [u8]) -> Result<(), RandomSourceError> {
        Err(RandomSourceError::new("injected entropy failure"))
    }
}

fn timestamp() -> UtcInstant {
    UtcInstant::parse_rfc3339("2026-07-21T14:00:00Z").unwrap()
}

fn small_manifest(rate: u16) -> DeckManifest {
    let reading: serde_json::Value = serde_json::from_str(READING_FIXTURE).unwrap();
    let mut deck = reading["deck"].clone();
    deck["reversal_rate_basis_points"] = rate.into();
    DeckManifest::from_json(&serde_json::to_string(&deck).unwrap()).unwrap()
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

#[test]
fn pinned_seed_produces_the_versioned_fixture_order() {
    let fixture: AlgorithmFixture = serde_json::from_str(ALGORITHM_FIXTURE).unwrap();
    assert_eq!(fixture.algorithm, ALGORITHM_ID);
    assert_eq!(fixture.algorithm_version, ALGORITHM_VERSION);
    assert_eq!(fixture.test_source, "sha256_counter_v1");
    let mut source = SeededSource::new(decode_hex(&fixture.seed_hex));
    let shuffled = shuffle_with_source(&small_manifest(5_000), timestamp(), &mut source, None)
        .expect("deterministic shuffle");
    let actual: Vec<_> = shuffled
        .cards()
        .iter()
        .map(|card| ExpectedCard {
            deck_card_id: card.deck_card_id().to_string(),
            orientation: card.orientation(),
        })
        .collect();

    assert_eq!(actual, fixture.expected);
}

#[test]
fn a_supplied_test_seed_is_reproducible() {
    let deck = DeckManifest::from_json(DECK_FIXTURE).unwrap();
    let seed = b"reproducible-test-seed".to_vec();
    let first = shuffle_with_source(
        &deck,
        timestamp(),
        &mut SeededSource::new(seed.clone()),
        Some(seed_commitment(&seed)),
    )
    .unwrap();
    let second = shuffle_with_source(
        &deck,
        timestamp(),
        &mut SeededSource::new(seed),
        Some(seed_commitment(b"reproducible-test-seed")),
    )
    .unwrap();

    assert_eq!(first, second);
}

#[test]
fn every_enabled_physical_card_appears_exactly_once() {
    let mut value: serde_json::Value = serde_json::from_str(DECK_FIXTURE).unwrap();
    for (index, card) in value["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        if index % 5 == 0 {
            card["enabled"] = false.into();
        }
    }
    value["reversal_rate_basis_points"] = 0.into();
    let deck = DeckManifest::from_json(&serde_json::to_string(&value).unwrap()).unwrap();
    let enabled: BTreeSet<_> = deck
        .enabled_cards()
        .map(|card| card.id().to_string())
        .collect();
    let mut source = SeededSource::new(vec![7; 32]);
    let shuffled = shuffle_with_source(&deck, timestamp(), &mut source, None).unwrap();
    let actual: BTreeSet<_> = shuffled
        .cards()
        .iter()
        .map(|card| card.deck_card_id().to_string())
        .collect();

    assert_eq!(shuffled.cards().len(), enabled.len());
    assert_eq!(actual, enabled);
    assert!(
        shuffled
            .cards()
            .iter()
            .all(|card| card.orientation() == Orientation::Upright)
    );
}

#[test]
fn reversal_boundaries_and_intermediate_rate_are_exact() {
    let mut zero_source = SeededSource::new(vec![0; 32]);
    let zero =
        shuffle_with_source(&small_manifest(0), timestamp(), &mut zero_source, None).unwrap();
    assert!(
        zero.cards()
            .iter()
            .all(|card| card.orientation() == Orientation::Upright)
    );

    let mut full_source = SeededSource::new(vec![0; 32]);
    let full =
        shuffle_with_source(&small_manifest(10_000), timestamp(), &mut full_source, None).unwrap();
    assert!(
        full.cards()
            .iter()
            .all(|card| card.orientation() == Orientation::Reversed)
    );

    let mut source = SequenceSource {
        values: vec![0, 0, 4_999, 5_000, 9_999],
        index: 0,
    };
    let intermediate =
        shuffle_with_source(&small_manifest(5_000), timestamp(), &mut source, None).unwrap();
    let orientations: Vec<_> = intermediate
        .cards()
        .iter()
        .map(|card| card.orientation())
        .collect();
    assert_eq!(
        orientations,
        vec![
            Orientation::Reversed,
            Orientation::Upright,
            Orientation::Upright
        ]
    );
}

#[test]
fn provenance_captures_algorithm_population_policy_and_source() {
    let deck = small_manifest(5_000);
    let seed = b"auditable-seed";
    let commitment = seed_commitment(seed);
    let shuffled = shuffle_with_source(
        &deck,
        timestamp(),
        &mut SeededSource::new(seed.to_vec()),
        Some(commitment.clone()),
    )
    .unwrap();

    match shuffled.provenance() {
        DrawProvenance::SoftwareShuffle {
            algorithm,
            algorithm_version,
            randomness_source,
            draw_manifest_id,
            enabled_card_count,
            reversal_policy,
            shuffled_at,
            seed_commitment,
        } => {
            assert_eq!(algorithm.as_str(), ALGORITHM_ID);
            assert_eq!(algorithm_version.get(), ALGORITHM_VERSION);
            assert_eq!(*randomness_source, RandomnessSource::Injected);
            assert_eq!(draw_manifest_id.as_str(), deck.draw_manifest_id().unwrap());
            assert_eq!(enabled_card_count.get(), 3);
            assert_eq!(reversal_policy.rate_basis_points(), 5_000);
            assert_eq!(*shuffled_at, timestamp());
            assert_eq!(seed_commitment.as_ref(), Some(&commitment));
        }
        DrawProvenance::Manual { .. } => panic!("expected software provenance"),
    }
}

#[test]
fn production_entrypoint_uses_os_randomness_and_preserves_the_population() {
    let deck = small_manifest(0);
    let shuffled = shuffle(&deck, timestamp()).unwrap();
    assert_eq!(shuffled.cards().len(), 3);
    assert!(matches!(
        shuffled.provenance(),
        DrawProvenance::SoftwareShuffle {
            randomness_source: RandomnessSource::OperatingSystem,
            ..
        }
    ));
}

#[test]
fn entropy_failure_returns_no_partial_shuffle() {
    let result = shuffle_with_source(
        &small_manifest(5_000),
        timestamp(),
        &mut FailingSource,
        None,
    );
    assert_eq!(result.unwrap_err().to_string(), "injected entropy failure");
}

#[test]
fn seed_commitment_does_not_expose_seed_bytes() {
    assert_eq!(
        seed_commitment(b"").as_str(),
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
