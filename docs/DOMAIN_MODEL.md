# Domain model proposal

Status: design input for schema v1; names and field shapes are not yet stable

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
- identity ID (stable, normalized, and optionally namespaced)
- taxonomy: conventional tarot | extension
- conventional classification, when applicable:
  - major: conventional number/identifier
  - minor: conventional suit and rank

DeckManifest
- manifest ID and manifest schema version
- name and optional creator/publisher metadata
- system or tradition as descriptive metadata, not behavior
- ordered DeckCard entries
- enabled-card set
- reversal policy/default
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

Manifest validation rejects duplicate deck-card IDs and canonical identities,
invalid enabled references, an empty enabled deck, and ambiguous mappings.
Manifest hashing uses a canonical, versioned representation and covers the
exact enabled card population relevant to a draw.

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

Fixed spreads require exactly one placement for every declared position.
Freeform readings snapshot their ordered positions so later edits to a saved
spread template cannot rewrite historical meaning. One-card and three-card
spreads are metadata-only built-ins with original wording; the three-card
position labels should be selected explicitly by the caller rather than
assuming a universal past/present/future meaning.

## Reading document

```text
TarotReading
- reading ID and artifact schema version
- optional opaque subject/session references supplied by the caller
- deck-manifest snapshot or content-addressed manifest reference
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

Follow-ups should be append-oriented records with their own IDs and timestamps,
not one lossy text field. The artifact layer can canonicalize the complete
document and compute a `sha256:` content ID. IDs and timestamps are caller
inputs or provider outputs, never synthesized during deserialization.

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
