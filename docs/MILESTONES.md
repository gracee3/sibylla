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

Status: complete on 2026-07-21.

- Implement extensible card identities and the conventional tarot profile.
- Implement deck/card manifests, enabled-card selection, reversal policy, and
  deterministic manifest hashing.
- Define strict versioned JSON fixtures and validation tests.

Exit: an artwork-free deck manifest can be created, imported, validated,
serialized, and reopened without loss.

## Phase 2: spreads and manual readings

Status: complete on 2026-07-21.

- Implement fixed and freeform spread definitions and snapshot semantics.
- Supply original one-card and configurable three-card definitions.
- Implement ordered placements and all three orientation states.
- Implement portable reading documents and follow-up annotations.

Exit: a physical reading can be recorded and recovered with the exact deck,
positions, printed labels, orientations, context, and interpretation intact.

## Phase 3: secure software draws

- Add a separate `sibylla-shuffle` crate with operating-system entropy from
  `getrandom` or `OsRng` and a versioned unbiased Fisher-Yates implementation.
- Shuffle the exact enabled population identified by the Phase 1 draw-manifest
  ID and generate reversals independently from its basis-point rate.
- Extend draw provenance with algorithm/version, randomness-source category,
  population ID, draw timestamp, and optional seed commitment.
- Add deterministic injected RNG tests, permutation/property tests, and pinned
  algorithm fixtures. Deterministic entropy must never be the production
  default.

Exit: software draws are unbiased by construction, auditable, reproducible in
tests, and never default to deterministic entropy in production.

## Phase 4: stable artifacts and consumer hand-off

- Add `sibylla-artifacts` and finalize canonical deck and reading envelopes and
  content IDs.
- Document compatibility and migration policy.
- Add consumer contract tests that do not require Oracle Studio in this repo.
- Publish a pinned Git revision that Oracle Studio can consume without a sibling
  path dependency.

Exit: the complete first tarot milestone can be implemented by an offline,
encrypted caller without changes to Sibylla's core contract.

## Phase 5: Oracle Studio MVP

This phase belongs to the independent Oracle Studio repository after Astraeus
and Sibylla expose usable contracts.

- Implement people and professional-client profiles and cross-domain sessions.
- Store pinned Astraeus artifacts and Sibylla reading artifacts in an encrypted
  local vault with key management.
- Add backup, import, export, permanent deletion, and journal search.
- Add source-linked editable memory and a basic offline UI.

Exit: a user can complete and recover the full encrypted physical-reading
workflow locally, with secure software draws available as an alternative.

## Later, separate checkpoints

- local deck-specific card recognition with encrypted reference images and user
  confirmation;
- AI interpretation that cites exact placements and disclosed memory inputs;
- optional accounts and synchronization;
- curated or licensed commercial deck packs and publisher partnerships.

These must not delay or leak into the first four phases.
