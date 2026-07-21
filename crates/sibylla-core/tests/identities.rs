use std::collections::BTreeSet;

use sibylla_core::{
    Arcana, CardIdentity, ConventionalCard, MajorArcana, MinorRank, MinorSuit, StableId,
};

#[test]
fn conventional_profile_contains_exactly_78_unique_identities() {
    let cards: Vec<_> = ConventionalCard::all().collect();
    let ids: BTreeSet<_> = cards.iter().map(|card| card.stable_id()).collect();

    assert_eq!(cards.len(), 78);
    assert_eq!(ids.len(), 78);
    assert_eq!(
        cards
            .iter()
            .filter(|card| card.arcana() == Arcana::Major)
            .count(),
        22
    );
    assert_eq!(
        cards
            .iter()
            .filter(|card| card.arcana() == Arcana::Minor)
            .count(),
        56
    );
}

#[test]
fn minor_profile_covers_every_suit_and_rank_pair() {
    let expected: BTreeSet<_> = MinorSuit::ALL
        .into_iter()
        .flat_map(|suit| {
            MinorRank::ALL
                .into_iter()
                .map(move |rank| ConventionalCard::Minor { suit, rank })
        })
        .collect();
    let actual: BTreeSet<_> = ConventionalCard::all()
        .filter(|card| card.arcana() == Arcana::Minor)
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn major_identity_is_semantic_not_numeric() {
    let strength = ConventionalCard::Major(MajorArcana::Strength);
    let justice = ConventionalCard::Major(MajorArcana::Justice);

    assert_eq!(strength.stable_id(), "strength");
    assert_eq!(justice.stable_id(), "justice");
    assert_eq!("strength".parse(), Ok(strength));
    assert_eq!("justice".parse(), Ok(justice));
}

#[test]
fn conventional_and_extension_wire_forms_are_distinct_and_strict() {
    let conventional = CardIdentity::Conventional(
        "seven_of_cups"
            .parse::<ConventionalCard>()
            .expect("known conventional identity"),
    );
    assert_eq!(
        serde_json::to_string(&conventional).unwrap(),
        r#"{"kind":"conventional","id":"seven_of_cups"}"#
    );

    let extension = CardIdentity::Extension {
        namespace: StableId::new("namespace", "org_example_tarot").unwrap(),
        id: StableId::new("extension_id", "threshold").unwrap(),
    };
    let json = serde_json::to_string(&extension).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"extension","namespace":"org_example_tarot","id":"threshold"}"#
    );
    assert_eq!(
        serde_json::from_str::<CardIdentity>(&json).unwrap(),
        extension
    );

    assert!(
        serde_json::from_str::<CardIdentity>(
            r#"{"kind":"extension","namespace":"org_example_tarot","id":"threshold","extra":true}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<CardIdentity>(r#"{"kind":"conventional","id":"unknown"}"#).is_err()
    );
}
