# Domain model

Status: Phase 2 spread and manual-reading schema implemented

## Design principles

- Identity is separate from a deck's printed title, numbering, suit, court
  names, correspondences, and artwork.
- A deck manifest describes the exact enabled cards being used. Code must not
  silently assume 78 cards, Rider-Waite-Smith naming, reversals, or four
  conventional suits.
- Ordered data remains ordered in memory and serialization. Stable IDs, not
  display labels or array offsets, connect records.
- All external data is untrusted. Unknown fields, duplicate IDs, dangling
  references, impossible placement order, and unsupported versions fail
  explicitly.

## Identity and deck metadata

```text
CardIdentity
- conventional: semantic stable ID with major or minor classification
- extension: explicit namespace and extension-local stable ID
- conventional majors have no universal number; numbering is deck-specific

DeckManifest
- manifest ID and manifest schema version
- name and optional creator/publisher metadata
- system or tradition as descriptive metadata, not behavior
- ordered DeckCard entries
- enabled-card set
- reversal-rate default in basis points (0 through 10,000)
- rights metadata; artwork references are optional and external

DeckCard
- deck-local card ID
- canonical CardIdentity
- printed title
- optional printed number, suit, and rank/court labels
- optional user-authored correspondences and notes
- optional caller-managed asset reference; never embedded by default
```

`seven_of_cups`, `temperance`, and `queen_of_swords` are conventional stable
IDs, not mandatory printed names. Nonconventional cards use an explicit
extension identity rather than being forced into a false conventional mapping.

Manifest validation rejects duplicate deck-card IDs, an empty card list, an
entirely disabled deck, blank supplied metadata, and duplicate correspondence
keys. Canonical identity duplicates are deliberately allowed when distinct
physical deck-card IDs identify variants.

Schema-v1 manifests use strict JSON and reject unknown fields. The full content
ID hashes canonical compact JSON for every field. The draw-manifest ID hashes a
separate versioned projection containing the manifest ID, reversal rate, and
ordered enabled cards with their deck-card and canonical identities. Notes,
attribution, rights, disabled cards, and asset references do not affect the
draw-manifest ID.

## Spreads and placements

```text
SpreadDefinition
- spread ID and schema version
- name
- layout: fixed | freeform
- ordered position definitions

SpreadPosition
- stable position ID
- display label
- optional meaning/prompt
- optional layout hint (nonsemantic coordinates)

Placement
- position ID
- position-label snapshot
- canonical card identity
- deck-card ID
- orientation: upright | reversed | unspecified
- draw order
- optional notes
```

Layout hints are finite, nonsemantic coordinates. All spread definitions contain
at least one position and reject duplicate position IDs. The original one-card
built-in uses the neutral label `Card`; the three-card constructor requires the
caller to supply all three labels and meanings rather than assuming a universal
past/present/future interpretation.

Fixed readings require exactly one placement for every declared position.
Freeform readings may use an ordered subset. Every placement must reference an
enabled physical deck card and a declared spread position, and its snapshotted
identity, printed title, and position label must match. A physical deck-card ID
and position ID may each appear only once. Draw order is the contiguous,
one-based range from 1 through the placement count.

## Reading document

```text
TarotReading
- reading ID and reading schema version
- optional opaque subject/session references supplied by the caller
- complete deck-manifest snapshot
- spread-definition snapshot
- optional question or intention
- optional background/context
- ordered placements
- draw provenance
- reader notes
- user interpretation
- follow-up/outcome annotations
- created and modified timestamps supplied by the caller
```

Snapshots preserve what the reader actually saw: printed titles, position
labels, manifest revision, and enabled deck. References alone are insufficient
because a mutable deck or spread template could otherwise change history.

Follow-ups are append-oriented annotation or outcome records with their own IDs
and timestamps, not one lossy text field. They must be unique, ordered, and fall
within the reading's created/modified timeline. Reading timestamps normalize to
UTC but are never synthesized during deserialization. Revisions revalidate all
placements and cannot move the modified timestamp backward.

Reading schema v1 uses strict JSON and rejects unknown fields throughout the
nested deck, spread, placement, provenance, and follow-up records. Stable
reading content IDs are deferred to the artifact checkpoint.

## Draw provenance

```text
DrawProvenance
- method: manual | software_shuffle
- algorithm ID and version, when software generated
- randomness source category
- deck-manifest content hash
- enabled-card count
- draw timestamp supplied by the caller
- reversal policy snapshot
- optional seed commitment or encrypted-seed reference
```

Phase 2 implements strict manual provenance with a caller-supplied
`recorded_at` timestamp. The software-shuffle variant and its additional fields
are reserved for Phase 3.

The normal production API obtains entropy from the operating system and runs
Fisher-Yates across the exact enabled card list. Index sampling must use
rejection sampling or a vetted unbiased API; `% range` is prohibited when it
introduces modulo bias. Orientation is sampled independently according to a
validated policy, and disabled reversals consume no semantic reversal result.

Raw seeds are sensitive and are not stored by default. A commitment can support
later audit without revealing the seed; encrypted seed storage belongs to the
application vault. A deterministic injected RNG is public enough for tests and
replays but is never the default constructor.

## Required invariant tests

- all enabled cards appear exactly once in every complete shuffled order;
- no disabled card appears and no card is lost or duplicated;
- a fixed test RNG/seed produces a pinned order for each algorithm version;
- zero, full, and intermediate reversal policies obey their contract;
- manual placement supports upright, reversed, and unspecified orientation;
- malformed manifests, spreads, readings, and artifacts fail on both
  construction and deserialization;
- JSON round trips preserve ordering and snapshots;
- unsupported schema and shuffle-algorithm versions fail explicitly;
- canonical serialization and content IDs are stable for checked-in fixtures.
