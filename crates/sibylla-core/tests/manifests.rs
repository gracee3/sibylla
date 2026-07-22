use sibylla_core::{
    Attribution, CardIdentity, ConventionalCard, Correspondence, DeckCard, DeckManifest,
    MajorArcana, ManifestError, ReversalPolicy, RightsMetadata, StableId, ValidationError,
};

const FIXTURE: &str = include_str!("../../../fixtures/decks/conventional-78-v1.json");
const EXTENSION_FIXTURE: &str = include_str!("../../../fixtures/decks/extension-tradition-v1.json");

fn id(field: &'static str, value: &str) -> StableId {
    StableId::new(field, value).unwrap()
}

fn card(
    deck_id: &str,
    identity: CardIdentity,
    printed_number: Option<&str>,
    enabled: bool,
) -> DeckCard {
    DeckCard::new(
        id("deck_card_id", deck_id),
        identity,
        deck_id.replace('_', " "),
        printed_number.map(str::to_owned),
        None,
        None,
        enabled,
        None,
        Vec::new(),
        None,
    )
    .unwrap()
}

fn manifest(cards: Vec<DeckCard>) -> Result<DeckManifest, ValidationError> {
    DeckManifest::new(
        id("deck_id", "test_deck"),
        "Test Deck",
        Attribution::default(),
        None,
        RightsMetadata::default(),
        ReversalPolicy::new(5_000).unwrap(),
        cards,
    )
}

fn fixture_value() -> serde_json::Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn parse_value(value: &serde_json::Value) -> DeckManifest {
    DeckManifest::from_json(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn conventional_fixture_round_trips_without_loss() {
    let deck = DeckManifest::from_json(FIXTURE).unwrap();

    assert_eq!(deck.id().as_str(), "sibylla_conventional_78");
    assert_eq!(deck.cards().len(), 78);
    assert_eq!(deck.enabled_cards().count(), 78);
    assert_eq!(deck.reversal_policy().rate_basis_points(), 5_000);
    assert!(deck.cards().iter().all(|card| card.asset_id().is_none()));
    assert_eq!(
        DeckManifest::from_json(&deck.to_json().unwrap()).unwrap(),
        deck
    );
    assert_eq!(
        serde_json::from_str::<DeckManifest>(&deck.to_pretty_json().unwrap()).unwrap(),
        deck
    );
}

#[test]
fn extension_tradition_fixture_preserves_non_rws_metadata() {
    let manifest = DeckManifest::from_json(EXTENSION_FIXTURE).unwrap();
    assert_eq!(manifest.cards().len(), 2);
    assert_eq!(manifest.cards()[0].printed_number(), Some("A"));
    assert_eq!(manifest.cards()[0].printed_suit(), Some("Gates"));
    assert!(matches!(
        manifest.cards()[0].identity(),
        CardIdentity::Extension { namespace, id }
            if namespace.as_str() == "org_example_threshold" && id.as_str() == "gate"
    ));
    assert!(matches!(
        manifest.cards()[1].identity(),
        CardIdentity::Extension { id, .. } if id.as_str() == "echo"
    ));
    assert_eq!(
        DeckManifest::from_json(&manifest.to_pretty_json().unwrap()).unwrap(),
        manifest
    );
}

#[test]
fn major_numbering_and_physical_variants_are_deck_specific() {
    let strength = CardIdentity::Conventional(ConventionalCard::Major(MajorArcana::Strength));
    let deck = manifest(vec![
        card("strength_eight", strength.clone(), Some("VIII"), true),
        card("strength_eleven", strength, Some("XI"), true),
    ])
    .unwrap();

    assert_eq!(deck.cards()[0].printed_number(), Some("VIII"));
    assert_eq!(deck.cards()[1].printed_number(), Some("XI"));
    assert_eq!(deck.cards()[0].identity(), deck.cards()[1].identity());
}

#[test]
fn constructors_reject_invalid_manifest_invariants() {
    assert_eq!(
        manifest(Vec::new()).unwrap_err(),
        ValidationError::EmptyDeck
    );

    let fool = CardIdentity::Conventional(ConventionalCard::Major(MajorArcana::Fool));
    assert_eq!(
        manifest(vec![card("fool", fool.clone(), None, false)]).unwrap_err(),
        ValidationError::NoEnabledCards
    );
    assert_eq!(
        manifest(vec![
            card("fool", fool.clone(), None, true),
            card("fool", fool, None, true),
        ])
        .unwrap_err(),
        ValidationError::DuplicateDeckCardId(id("deck_card_id", "fool"))
    );
    assert_eq!(
        ReversalPolicy::new(10_001).unwrap_err(),
        ValidationError::InvalidReversalRate(10_001)
    );
}

#[test]
fn correspondence_keys_are_ordered_and_unique() {
    let key = id("correspondence_key", "element");
    let correspondences = vec![
        Correspondence::new(key.clone(), "water").unwrap(),
        Correspondence::new(key.clone(), "air").unwrap(),
    ];
    let result = DeckCard::new(
        id("deck_card_id", "moon"),
        CardIdentity::Conventional(ConventionalCard::Major(MajorArcana::Moon)),
        "The Moon",
        None,
        None,
        None,
        true,
        None,
        correspondences,
        None,
    );

    assert_eq!(
        result.unwrap_err(),
        ValidationError::DuplicateCorrespondenceKey(key)
    );
}

#[test]
fn deserialization_rejects_unknown_fields_blank_metadata_and_bad_versions() {
    let mut value = fixture_value();
    value["unexpected"] = true.into();
    assert!(parse_manifest_error(&value));

    let mut value = fixture_value();
    value["cards"][0]["unexpected"] = true.into();
    assert!(parse_manifest_error(&value));

    let mut value = fixture_value();
    value["attribution"]["artist"] = "  ".into();
    assert!(parse_manifest_error(&value));

    let mut value = fixture_value();
    value["schema_version"] = 2.into();
    assert!(matches!(
        DeckManifest::from_json(&serde_json::to_string(&value).unwrap()),
        Err(ManifestError::UnsupportedSchema(2))
    ));
}

fn parse_manifest_error(value: &serde_json::Value) -> bool {
    DeckManifest::from_json(&serde_json::to_string(value).unwrap()).is_err()
}

#[test]
fn full_and_draw_hashes_cover_their_intended_inputs() {
    let original_value = fixture_value();
    let original = parse_value(&original_value);
    let content_id = original.content_id().unwrap();
    let draw_id = original.draw_manifest_id().unwrap();

    for mutate in [
        |value: &mut serde_json::Value| value["cards"][0]["notes"] = "reader note".into(),
        |value: &mut serde_json::Value| value["attribution"]["artist"] = "Local artist".into(),
        |value: &mut serde_json::Value| value["rights"]["notes"] = "Updated rights note".into(),
        |value: &mut serde_json::Value| value["cards"][0]["asset_id"] = "local_asset".into(),
    ] {
        let mut value = original_value.clone();
        mutate(&mut value);
        let changed = parse_value(&value);
        assert_ne!(changed.content_id().unwrap(), content_id);
        assert_eq!(changed.draw_manifest_id().unwrap(), draw_id);
    }

    let mut disabled = original_value.clone();
    disabled["cards"][0]["enabled"] = false.into();
    assert_ne!(parse_value(&disabled).draw_manifest_id().unwrap(), draw_id);

    let mut reordered = original_value.clone();
    reordered["cards"].as_array_mut().unwrap().swap(0, 1);
    assert_ne!(parse_value(&reordered).draw_manifest_id().unwrap(), draw_id);

    let mut remapped = original_value.clone();
    remapped["cards"][0]["identity"] = serde_json::json!({
        "kind": "extension",
        "namespace": "org_sibylla_fixture",
        "id": "chariot_variant"
    });
    assert_ne!(parse_value(&remapped).draw_manifest_id().unwrap(), draw_id);

    let mut reversal = original_value;
    reversal["reversal_rate_basis_points"] = 0.into();
    assert_ne!(parse_value(&reversal).draw_manifest_id().unwrap(), draw_id);
}

#[test]
fn fixture_hashes_are_stable() {
    let deck = DeckManifest::from_json(FIXTURE).unwrap();
    assert_eq!(
        deck.content_id().unwrap(),
        "sha256:82a425e9fa4765fa25ce345581da9a9f53e1fab8d3fb994441e775fe54f95461"
    );
    assert_eq!(
        deck.draw_manifest_id().unwrap(),
        "sha256:3b8a1abe36e74ff5a5686464d6478e4d58c71b14e7121677ba096b6b016a6a45"
    );
}
