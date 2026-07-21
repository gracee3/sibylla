//! Exercises only the public API available to an independent application.

use sibylla_artifacts::{Artifact, ArtifactKind, ReadingArtifact};
use sibylla_core::TarotReading;

const READING_FIXTURE: &str = include_str!("../../../fixtures/readings/manual-three-card-v1.json");

#[test]
fn an_offline_consumer_can_export_identify_and_recover_a_reading() {
    let reading = TarotReading::from_json(READING_FIXTURE).unwrap();
    let exported = ReadingArtifact::new(reading).to_json().unwrap();

    let imported = Artifact::from_json(&exported).unwrap();
    assert_eq!(imported.kind(), ArtifactKind::Reading);
    assert!(
        imported
            .content_id()
            .unwrap()
            .as_str()
            .starts_with("sha256:")
    );

    let Artifact::Reading(imported) = imported else {
        unreachable!("kind was checked above")
    };
    assert_eq!(imported.payload().id().as_str(), "phase_two_manual_reading");
    assert_eq!(imported.payload().placements().len(), 3);
}
