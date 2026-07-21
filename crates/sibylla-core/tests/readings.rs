use sibylla_core::{
    FollowUp, FollowUpKind, Orientation, ReadingError, TarotReading, UtcInstant, ValidationError,
};

const FIXTURE: &str = include_str!("../../../fixtures/readings/manual-three-card-v1.json");

fn fixture_value() -> serde_json::Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn parse_value(value: &serde_json::Value) -> Result<TarotReading, ReadingError> {
    TarotReading::from_json(&serde_json::to_string(value).unwrap())
}

fn assert_invalid(value: &serde_json::Value, expected: ValidationError) {
    match parse_value(value) {
        Err(ReadingError::InvalidReading(actual)) => assert_eq!(actual, expected),
        other => panic!("expected invalid reading {expected:?}, got {other:?}"),
    }
}

#[test]
fn manual_reading_fixture_round_trips_with_complete_snapshots() {
    let reading = TarotReading::from_json(FIXTURE).unwrap();

    assert_eq!(reading.id().as_str(), "phase_two_manual_reading");
    assert_eq!(reading.deck().cards().len(), 3);
    assert_eq!(reading.spread().positions().len(), 3);
    assert_eq!(reading.placements().len(), 3);
    assert_eq!(reading.placements()[0].orientation(), Orientation::Upright);
    assert_eq!(reading.placements()[1].orientation(), Orientation::Reversed);
    assert_eq!(
        reading.placements()[2].orientation(),
        Orientation::Unspecified
    );
    assert_eq!(reading.follow_ups().len(), 1);
    assert_eq!(
        TarotReading::from_json(&reading.to_pretty_json().unwrap()).unwrap(),
        reading
    );
}

#[test]
fn freeform_readings_may_use_an_ordered_subset_of_declared_positions() {
    let mut value = fixture_value();
    value["spread"]["layout"] = "freeform".into();
    value["placements"].as_array_mut().unwrap().truncate(1);
    let reading = parse_value(&value).unwrap();

    assert_eq!(reading.placements().len(), 1);
    assert_eq!(reading.placements()[0].draw_order(), 1);
}

#[test]
fn fixed_spreads_require_every_position_exactly_once() {
    let mut value = fixture_value();
    value["placements"].as_array_mut().unwrap().pop();
    assert_invalid(&value, ValidationError::IncompleteFixedSpread);

    let mut value = fixture_value();
    value["placements"][1]["position_id"] = "situation".into();
    value["placements"][1]["position_label"] = "Situation".into();
    assert_invalid(
        &value,
        ValidationError::DuplicatePlacementPositionId(
            sibylla_core::StableId::new("position_id", "situation").unwrap(),
        ),
    );
}

#[test]
fn placements_must_match_the_snapshotted_spread_and_deck() {
    let mut value = fixture_value();
    value["placements"][0]["position_id"] = "missing".into();
    assert_invalid(
        &value,
        ValidationError::UnknownSpreadPosition(
            sibylla_core::StableId::new("position_id", "missing").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["placements"][0]["position_label"] = "Changed".into();
    assert_invalid(
        &value,
        ValidationError::PositionLabelMismatch(
            sibylla_core::StableId::new("position_id", "situation").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["placements"][0]["deck_card_id"] = "missing".into();
    assert_invalid(
        &value,
        ValidationError::UnknownDeckCard(
            sibylla_core::StableId::new("deck_card_id", "missing").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["deck"]["cards"][0]["enabled"] = false.into();
    assert_invalid(
        &value,
        ValidationError::DisabledDeckCard(
            sibylla_core::StableId::new("deck_card_id", "chariot").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["placements"][0]["card_identity"]["id"] = "strength".into();
    assert_invalid(
        &value,
        ValidationError::CardIdentityMismatch(
            sibylla_core::StableId::new("deck_card_id", "chariot").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["placements"][0]["printed_title"] = "Changed".into();
    assert_invalid(
        &value,
        ValidationError::PrintedTitleMismatch(
            sibylla_core::StableId::new("deck_card_id", "chariot").unwrap(),
        ),
    );
}

#[test]
fn physical_cards_and_draw_orders_cannot_be_duplicated() {
    let mut value = fixture_value();
    value["placements"][1]["deck_card_id"] = "chariot".into();
    value["placements"][1]["card_identity"]["id"] = "chariot".into();
    value["placements"][1]["printed_title"] = "The Chariot".into();
    assert_invalid(
        &value,
        ValidationError::DuplicatePlacedDeckCard(
            sibylla_core::StableId::new("deck_card_id", "chariot").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["placements"][0]["draw_order"] = 2.into();
    assert_invalid(&value, ValidationError::InvalidDrawOrder);
}

#[test]
fn reading_and_nested_wire_shapes_are_strict() {
    let mut value = fixture_value();
    value["unexpected"] = true.into();
    assert!(matches!(parse_value(&value), Err(ReadingError::Json(_))));

    let mut value = fixture_value();
    value["placements"][0]["unexpected"] = true.into();
    assert!(matches!(parse_value(&value), Err(ReadingError::Json(_))));

    let mut value = fixture_value();
    value["schema_version"] = 2.into();
    assert!(matches!(
        parse_value(&value),
        Err(ReadingError::UnsupportedSchema(2))
    ));

    let mut value = fixture_value();
    value["question"] = " ".into();
    assert_invalid(
        &value,
        ValidationError::EmptyText {
            field: "reading.question",
        },
    );
}

#[test]
fn reading_timeline_and_follow_up_order_are_validated() {
    let mut value = fixture_value();
    value["draw_provenance"]["recorded_at"] = "2026-07-20T14:00:00Z".into();
    assert_invalid(
        &value,
        ValidationError::InvalidTimestampOrder {
            field: "draw_provenance.timestamp",
        },
    );

    let mut value = fixture_value();
    let duplicate = value["follow_ups"][0].clone();
    value["follow_ups"].as_array_mut().unwrap().push(duplicate);
    assert_invalid(
        &value,
        ValidationError::DuplicateFollowUpId(
            sibylla_core::StableId::new("follow_up_id", "first_outcome").unwrap(),
        ),
    );

    let mut value = fixture_value();
    value["modified_at"] = "2026-07-23T14:00:00Z".into();
    value["follow_ups"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "earlier_annotation",
            "kind": "annotation",
            "content": "Earlier but appended later.",
            "created_at": "2026-07-21T15:00:00Z"
        }));
    assert_invalid(&value, ValidationError::FollowUpOrder);
}

#[test]
fn reopened_readings_can_be_revised_and_annotated() {
    let mut reading = TarotReading::from_json(FIXTURE).unwrap();
    let revised_at = UtcInstant::parse_rfc3339("2026-07-23T14:00:00Z").unwrap();
    reading
        .revise(
            Some("A revised fictional question?".into()),
            reading.context().map(str::to_owned),
            reading.placements().to_vec(),
            Some("Revised reader notes.".into()),
            reading.interpretation().map(str::to_owned),
            revised_at,
        )
        .unwrap();
    let follow_up_at = UtcInstant::parse_rfc3339("2026-07-24T14:00:00Z").unwrap();
    reading
        .append_follow_up(
            FollowUp::new(
                sibylla_core::StableId::new("follow_up_id", "later_annotation").unwrap(),
                FollowUpKind::Annotation,
                "A later fictional annotation.",
                follow_up_at,
            )
            .unwrap(),
            follow_up_at,
        )
        .unwrap();

    assert_eq!(reading.question(), Some("A revised fictional question?"));
    assert_eq!(reading.reader_notes(), Some("Revised reader notes."));
    assert_eq!(reading.follow_ups().len(), 2);
    assert_eq!(reading.modified_at(), follow_up_at);
    assert_eq!(
        TarotReading::from_json(&reading.to_json().unwrap()).unwrap(),
        reading
    );
}

#[test]
fn caller_timestamps_are_normalized_to_utc() {
    let timestamp = UtcInstant::parse_rfc3339("2026-07-21T10:00:00-04:00").unwrap();
    assert_eq!(timestamp.to_rfc3339(), "2026-07-21T14:00:00Z");
    assert!(UtcInstant::parse_rfc3339("not-a-timestamp").is_err());
}

#[test]
fn software_provenance_must_match_the_embedded_deck_snapshot() {
    let mut value = fixture_value();
    let deck =
        sibylla_core::DeckManifest::from_json(&serde_json::to_string(&value["deck"]).unwrap())
            .unwrap();
    value["draw_provenance"] = serde_json::json!({
        "method": "software_shuffle",
        "algorithm": "fisher_yates",
        "algorithm_version": 1,
        "randomness_source": "operating_system",
        "draw_manifest_id": deck.draw_manifest_id().unwrap(),
        "enabled_card_count": 3,
        "reversal_policy": 5000,
        "shuffled_at": "2026-07-21T14:00:00Z",
        "seed_commitment": null
    });
    let reading = parse_value(&value).unwrap();
    assert!(matches!(
        reading.draw_provenance(),
        sibylla_core::DrawProvenance::SoftwareShuffle { .. }
    ));

    let mut wrong_id = value.clone();
    wrong_id["draw_provenance"]["draw_manifest_id"] = format!("sha256:{}", "00".repeat(32)).into();
    assert_invalid(&wrong_id, ValidationError::ProvenanceManifestMismatch);

    let mut wrong_count = value.clone();
    wrong_count["draw_provenance"]["enabled_card_count"] = 2.into();
    assert_invalid(&wrong_count, ValidationError::ProvenanceCardCountMismatch);

    let mut wrong_reversals = value;
    wrong_reversals["draw_provenance"]["reversal_policy"] = 0.into();
    assert_invalid(
        &wrong_reversals,
        ValidationError::ProvenanceReversalMismatch,
    );
}
