use sibylla_core::{
    LayoutHint, SpreadDefinition, SpreadError, SpreadLayout, SpreadPosition, StableId,
    ValidationError,
};

fn id(field: &'static str, value: &str) -> StableId {
    StableId::new(field, value).unwrap()
}

fn position(id_value: &str, label: &str, x: f64) -> SpreadPosition {
    SpreadPosition::new(
        id("position_id", id_value),
        label,
        None,
        Some(LayoutHint::new(x, 0.0).unwrap()),
    )
    .unwrap()
}

#[test]
fn one_card_builtin_is_original_fixed_metadata() {
    let spread = SpreadDefinition::one_card();

    assert_eq!(spread.id().as_str(), "one_card");
    assert_eq!(spread.name(), "One Card");
    assert_eq!(spread.layout(), SpreadLayout::Fixed);
    assert_eq!(spread.positions().len(), 1);
    assert_eq!(spread.positions()[0].id().as_str(), "card");
    assert_eq!(spread.positions()[0].label(), "Card");
    assert_eq!(spread.positions()[0].meaning(), None);
}

#[test]
fn three_card_spread_requires_caller_supplied_semantics() {
    let spread = SpreadDefinition::three_card(
        id("spread_id", "question_context_action"),
        "Question, Context, Action",
        [
            position("question", "Question", -1.0),
            position("context", "Context", 0.0),
            position("action", "Action", 1.0),
        ],
    )
    .unwrap();

    assert_eq!(spread.positions()[0].label(), "Question");
    assert_eq!(spread.positions()[2].label(), "Action");
    assert!(!spread.to_json().unwrap().contains("past"));
}

#[test]
fn spread_construction_and_deserialization_enforce_invariants() {
    assert_eq!(
        SpreadDefinition::new(
            id("spread_id", "empty"),
            "Empty",
            SpreadLayout::Freeform,
            Vec::new(),
        )
        .unwrap_err(),
        ValidationError::EmptySpread
    );

    let duplicate_id = id("position_id", "same");
    assert_eq!(
        SpreadDefinition::new(
            id("spread_id", "duplicate"),
            "Duplicate",
            SpreadLayout::Fixed,
            vec![
                position("same", "First", 0.0),
                position("same", "Second", 1.0)
            ],
        )
        .unwrap_err(),
        ValidationError::DuplicateSpreadPositionId(duplicate_id)
    );
    assert_eq!(
        LayoutHint::new(f64::NAN, 0.0).unwrap_err(),
        ValidationError::NonFinite { field: "layout.x" }
    );

    let mut value = serde_json::to_value(SpreadDefinition::one_card()).unwrap();
    value["unexpected"] = true.into();
    assert!(SpreadDefinition::from_json(&serde_json::to_string(&value).unwrap()).is_err());

    let mut value = serde_json::to_value(SpreadDefinition::one_card()).unwrap();
    value["positions"][0]["label"] = " ".into();
    assert!(SpreadDefinition::from_json(&serde_json::to_string(&value).unwrap()).is_err());

    let mut value = serde_json::to_value(SpreadDefinition::one_card()).unwrap();
    value["schema_version"] = 2.into();
    assert!(matches!(
        SpreadDefinition::from_json(&serde_json::to_string(&value).unwrap()),
        Err(SpreadError::UnsupportedSchema(2))
    ));
}

#[test]
fn spread_json_round_trip_preserves_order_meaning_and_layout() {
    let spread = SpreadDefinition::new(
        id("spread_id", "freeform_example"),
        "Freeform Example",
        SpreadLayout::Freeform,
        vec![
            SpreadPosition::new(
                id("position_id", "center"),
                "Center",
                Some("The focal point".into()),
                Some(LayoutHint::new(0.25, -0.5).unwrap()),
            )
            .unwrap(),
            position("crossing", "Crossing", 0.5),
        ],
    )
    .unwrap();
    let decoded = SpreadDefinition::from_json(&spread.to_pretty_json().unwrap()).unwrap();

    assert_eq!(decoded, spread);
    assert_eq!(decoded.positions()[0].id().as_str(), "center");
    assert_eq!(decoded.positions()[0].meaning(), Some("The focal point"));
    assert_eq!(decoded.positions()[0].layout_hint().unwrap().x(), 0.25);
}
