use sibylla_artifacts::{
    ARTIFACT_SCHEMA_VERSION, Artifact, ArtifactError, ArtifactKind, DeckArtifact, ReadingArtifact,
};
use sibylla_core::{DeckManifest, TarotReading, UtcInstant};

const DECK_FIXTURE: &str = include_str!("../../../fixtures/decks/conventional-78-v1.json");
const READING_FIXTURE: &str = include_str!("../../../fixtures/readings/manual-three-card-v1.json");
const DECK_GOLDEN_ID: &str = include_str!("../../../fixtures/artifacts/deck-envelope-v1.sha256");
const READING_GOLDEN_ID: &str =
    include_str!("../../../fixtures/artifacts/reading-envelope-v1.sha256");

fn deck_artifact() -> DeckArtifact {
    DeckArtifact::new(DeckManifest::from_json(DECK_FIXTURE).unwrap())
}

fn reading_artifact() -> ReadingArtifact {
    ReadingArtifact::new(TarotReading::from_json(READING_FIXTURE).unwrap())
}

#[test]
fn canonical_envelopes_have_pinned_content_ids() {
    assert_eq!(ARTIFACT_SCHEMA_VERSION, 1);
    assert_eq!(
        deck_artifact().content_id().unwrap().as_str(),
        DECK_GOLDEN_ID.trim()
    );
    assert_eq!(
        reading_artifact().content_id().unwrap().as_str(),
        READING_GOLDEN_ID.trim()
    );
}

#[test]
fn compact_and_pretty_deck_envelopes_reopen_canonically() {
    let artifact = deck_artifact();
    let compact = artifact.to_json().unwrap();
    let pretty = artifact.to_pretty_json().unwrap();

    assert_eq!(DeckArtifact::from_json(&compact).unwrap(), artifact);
    assert_eq!(DeckArtifact::from_json(&pretty).unwrap(), artifact);
    assert_eq!(
        DeckArtifact::from_json(&pretty).unwrap().to_json().unwrap(),
        compact
    );
    assert!(compact.starts_with(
        r#"{"schema_version":1,"artifact_type":"deck","payload":{"schema_version":1,"#
    ));
}

#[test]
fn reading_envelopes_preserve_the_complete_validated_reading() {
    let artifact = reading_artifact();
    let reopened = ReadingArtifact::from_json(&artifact.to_pretty_json().unwrap()).unwrap();

    assert_eq!(reopened, artifact);
    assert_eq!(reopened.payload().placements().len(), 3);
    assert_eq!(reopened.payload().follow_ups().len(), 1);
}

#[test]
fn generic_dispatch_reports_kind_and_preserves_canonical_bytes() {
    for expected in [
        Artifact::Deck(Box::new(deck_artifact())),
        Artifact::Reading(Box::new(reading_artifact())),
    ] {
        let canonical = expected.to_json().unwrap();
        let reopened = Artifact::from_json(&canonical).unwrap();
        assert_eq!(reopened.kind(), expected.kind());
        assert_eq!(reopened, expected);
        assert_eq!(reopened.to_json().unwrap(), canonical);
        assert_eq!(
            reopened.content_id().unwrap(),
            expected.content_id().unwrap()
        );
    }
}

#[test]
fn typed_readers_reject_the_other_artifact_kind() {
    assert!(matches!(
        DeckArtifact::from_json(&reading_artifact().to_json().unwrap()),
        Err(ArtifactError::UnexpectedType {
            expected: ArtifactKind::Deck,
            actual: ArtifactKind::Reading,
        })
    ));
    assert!(matches!(
        ReadingArtifact::from_json(&deck_artifact().to_json().unwrap()),
        Err(ArtifactError::UnexpectedType {
            expected: ArtifactKind::Reading,
            actual: ArtifactKind::Deck,
        })
    ));
}

#[test]
fn envelope_versions_and_unknown_fields_fail_explicitly() {
    let mut value: serde_json::Value =
        serde_json::from_str(&deck_artifact().to_json().unwrap()).unwrap();
    value["schema_version"] = 2.into();
    assert!(matches!(
        Artifact::from_json(&serde_json::to_string(&value).unwrap()),
        Err(ArtifactError::UnsupportedSchema(2))
    ));

    value["schema_version"] = 1.into();
    value["unexpected"] = true.into();
    assert!(matches!(
        Artifact::from_json(&serde_json::to_string(&value).unwrap()),
        Err(ArtifactError::Json(_))
    ));
}

#[test]
fn nested_payload_future_versions_fail_before_consumer_use() {
    let mut value: serde_json::Value =
        serde_json::from_str(&deck_artifact().to_json().unwrap()).unwrap();
    value["payload"]["schema_version"] = 2.into();
    assert!(matches!(
        Artifact::from_json(&serde_json::to_string(&value).unwrap()),
        Err(ArtifactError::Json(_))
    ));
}

#[test]
fn nested_domain_validation_cannot_be_bypassed() {
    let mut value: serde_json::Value =
        serde_json::from_str(&reading_artifact().to_json().unwrap()).unwrap();
    value["payload"]["placements"][0]["deck_card_id"] = "missing".into();
    assert!(matches!(
        Artifact::from_json(&serde_json::to_string(&value).unwrap()),
        Err(ArtifactError::Json(_))
    ));
}

#[test]
fn payload_changes_change_the_content_id() {
    let original = deck_artifact();
    let mut value: serde_json::Value = serde_json::from_str(DECK_FIXTURE).unwrap();
    value["name"] = "A deliberately changed fixture name".into();
    let changed = DeckArtifact::new(
        DeckManifest::from_json(&serde_json::to_string(&value).unwrap()).unwrap(),
    );

    assert_ne!(
        original.content_id().unwrap(),
        changed.content_id().unwrap()
    );
}

#[test]
fn reading_revisions_change_the_artifact_content_id() {
    let original = reading_artifact();
    let mut reading = original.clone().into_payload();
    let modified_at = UtcInstant::parse_rfc3339("2026-07-23T14:00:00Z").unwrap();
    reading
        .revise(
            Some("A revised fictional question?".into()),
            reading.context().map(str::to_owned),
            reading.placements().to_vec(),
            reading.reader_notes().map(str::to_owned),
            reading.interpretation().map(str::to_owned),
            modified_at,
        )
        .unwrap();
    let changed = ReadingArtifact::new(reading);

    assert_ne!(
        original.content_id().unwrap(),
        changed.content_id().unwrap()
    );
}

#[test]
fn duplicate_envelope_keys_are_rejected() {
    let json = deck_artifact().to_json().unwrap();
    let duplicated = json.replacen(
        r#"{"schema_version":1,"#,
        r#"{"schema_version":1,"schema_version":1,"#,
        1,
    );
    assert!(matches!(
        Artifact::from_json(&duplicated),
        Err(ArtifactError::Json(_))
    ));
}
