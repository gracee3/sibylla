# Milestones

## Phase 0: repository baseline

Status: complete on 2026-07-21. Crates remain unpublished until their public
contracts and schema compatibility policy are ready.

- Use AGPL-3.0-or-later and keep crates unpublished during contract design.
- Add pinned Rust toolchain, Cargo workspace, formatting, lint, docs, test, and
  dependency-policy CI aligned with Astraeus.
- Establish `sibylla-core` with explicit validation errors and no UI/database
  dependencies.
- Check in only original metadata fixtures with a provenance note.

Exit: a clean clone builds, tests, lints, and documents with locked dependencies.

## Phase 1: identities and deck manifests

- Implement extensible card identities and the conventional tarot profile.
- Implement deck/card manifests, enabled-card selection, reversal policy, and
  deterministic manifest hashing.
- Define strict versioned JSON fixtures and validation tests.

Exit: an artwork-free deck manifest can be created, imported, validated,
serialized, and reopened without loss.

## Phase 2: spreads and manual readings

- Implement fixed and freeform spread definitions and snapshot semantics.
- Supply original one-card and configurable three-card definitions.
- Implement ordered placements and all three orientation states.
- Implement portable reading documents and follow-up annotations.

Exit: a physical reading can be recorded and recovered with the exact deck,
positions, printed labels, orientations, context, and interpretation intact.

## Phase 3: secure software draws

- Add the operating-system entropy provider and versioned unbiased Fisher-Yates.
- Add independent reversal generation and complete draw provenance.
- Add deterministic injected RNG tests, permutation/property tests, and pinned
  algorithm fixtures.

Exit: software draws are unbiased by construction, auditable, reproducible in
tests, and never default to deterministic entropy in production.

## Phase 4: stable artifacts and consumer hand-off

- Finalize canonical deck and reading artifact envelopes and content IDs.
- Document compatibility and migration policy.
- Add consumer contract tests that do not require Oracle Studio in this repo.
- Publish a pinned Git revision that Oracle Studio can consume without a sibling
  path dependency.

Exit: the complete first tarot milestone can be implemented by an offline,
encrypted caller without changes to Sibylla's core contract.

## Later, separate checkpoints

- curated or licensed deck packs;
- editable, source-linked memory in Oracle Studio;
- local deck-specific card recognition with user confirmation;
- AI interpretation that cites exact placements and disclosed memory inputs.

These must not delay or leak into the first four phases.
