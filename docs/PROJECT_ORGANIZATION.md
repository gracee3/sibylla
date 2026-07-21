# Project organization

Status: Phase 4 stable artifact hand-off complete
Last reviewed: 2026-07-21

## Mission

Build a small, copyright-safe Rust tarot engine with explicit validation,
reproducible test seams, cryptographically secure production shuffling, and
versioned portable artifacts.

## Family boundary

```text
astraeus ─┐
          ├──> oracle-studio ──optional adapter──> magnolia
sibylla ──┘
```

The engines are peers, not dependencies of one another. Alignment with
Astraeus means matching engineering contracts where they help consumers:

- a headless core with validated domain values;
- deserialization that cannot bypass invariants;
- explicit errors and no partial-success records;
- deterministic injected test seams around nondeterministic providers;
- versioned, canonical serialized artifacts with content digests;
- provenance on externally produced results;
- pinned Rust tooling, formatting, lint, documentation, test, and dependency
  policy checks.

It does not mean sharing astrology types, creating a parent Cargo workspace, or
using committed sibling path dependencies.

## Proposed repository shape

```text
crates/
  sibylla-core/       card, deck, spread, placement, reading, validation
  sibylla-shuffle/    secure and injected deterministic shuffle providers
  sibylla-artifacts/  versioned canonical reading/deck envelopes and digests
docs/
fixtures/             original metadata-only fixtures with provenance
```

The three crates now reflect the frozen first-milestone boundaries: domain
validation, entropy-backed drawing, and stable interchange artifacts.

## Ownership decisions

### Sibylla owns

- deck-independent card identity and taxonomy;
- deck manifests and mappings from printed cards to identities;
- spread definitions and ordered positions;
- placements, orientation, and draw order;
- manual and software draw provenance;
- a portable tarot reading document, including the tarot question/context and
  user-authored interpretation fields needed to use the engine independently;
- schema validation, canonical serialization, and content IDs.

### Oracle Studio owns

- person and professional-client records;
- cross-domain session IDs and relationships;
- encrypted-at-rest vault implementation, key management, backups, import and
  permanent-deletion workflows;
- journal indexing, editable memory, practitioner visibility controls, and UI;
- composition with Astraeus artifacts and optional Magnolia capabilities.

A Sibylla artifact may carry opaque caller-provided association references, but
Sibylla neither defines nor resolves a person or client. Sensitive content can
be serialized by Sibylla, but encryption and persistence remain the caller's
responsibility. Public APIs and docs must not claim that Sibylla alone is an
encrypted journal.

## Deferred work

- camera recognition and image processing;
- AI interpretation or hidden conversational memory;
- accounts, synchronization, subscriptions, and hosted services;
- bundled commercial deck art, guidebooks, fonts, or recognition models;
- Oracle Studio and Magnolia adapters.

## Confirmed bootstrap decisions

1. Sibylla uses AGPL-3.0-or-later, matching Astraeus.
2. Portable readings include free-text context and notes so Sibylla remains
   independently useful; practitioner-only memory remains in Oracle Studio.
3. Schema v1 supports an extensible namespaced card identity with a validated
   conventional taxonomy profile.

## Phase 1 decisions

- Major Arcana identities are semantic. Printed numbers are deck-local metadata
  so decks may number Strength, Justice, or any other card differently.
- Deck-card IDs identify physical cards. Multiple physical cards may map to one
  canonical identity, supporting alternate variants within a deck.
- A reversal default is an exact rate in basis points from 0 through 10,000.
- The complete manifest content ID covers all serialized metadata. A separate
  draw-manifest ID covers only the ordered enabled population, its canonical
  mappings, the manifest ID, and the reversal rate.

## Phase 2 decisions

- Reading schema v1 embeds complete deck and spread snapshots so later template
  edits cannot rewrite the historical reading.
- Fixed readings place every declared position exactly once. Freeform readings
  may use an ordered subset of their explicitly declared positions.
- Placements use one-based contiguous draw order and snapshot the position
  label, canonical identity, deck-card ID, and printed title.
- Manual readings accept upright, reversed, and unspecified orientation. A
  manual provenance timestamp records when the physical layout was entered.
- Reading IDs and timestamps are caller-supplied. Optional subject and session
  references remain opaque strings owned by the calling application.
- Reopened readings can revise placements and text or append timestamped
  annotations and outcomes without bypassing snapshot or timeline validation.

## Phase 3 decisions

- `sibylla-shuffle::shuffle` is the production entrypoint and always obtains
  randomness from the operating system through `getrandom`.
- Fisher-Yates algorithm version 1 uses rejection-sampled little-endian `u64`
  values; simple biased modulo sampling is never used.
- Reversal sampling happens after card order and independently for each card.
  Zero and 10,000 basis-point policies require no reversal entropy.
- Injected sources are explicit and recorded as `injected`; they
  exist for tests and replays and are never selected by the default API.
- Software provenance records algorithm/version, randomness category,
  draw-manifest ID, enabled count, reversal policy, timestamp, and optional
  seed commitment. Reading validation cross-checks the population fields against
  the embedded deck snapshot.

## Phase 4 decisions

- `sibylla-artifacts` wraps validated deck and reading payloads in strict
  envelope schema version 1 without taking ownership of persistence or
  encryption.
- Artifact IDs hash the exact compact canonical envelope, including the
  envelope discriminator and every nested payload field.
- Pretty JSON is a presentation form. It must be reparsed and canonically
  serialized before calculating or verifying an ID.
- Envelope and nested schema versions are independent and both fail explicitly
  when unsupported. Version 1 never ignores unknown fields or silently migrates.
- Golden digests freeze deck and reading serialization, and a public-API-only
  contract test represents an independent Oracle Studio consumer.
