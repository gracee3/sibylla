# Sibylla roadmap

Sibylla's first four domain phases are complete: extensible card identities,
deck manifests, spreads and readings, secure software draws, and stable
artifact envelopes. The next work hardens those contracts before any camera or
AI feature is considered.

## Engine priorities

1. **Compatibility hardening**
   - Add pinned fixtures for deck and reading canonical JSON and content IDs.
   - Test revision semantics, follow-up ordering, and schema-version failures.
   - Publish consumer contract fixtures without depending on Oracle Studio.
   - Define explicit migration readers before changing serialized fields.

2. **Deck and spread completeness**
   - Exercise non-Rider–Waite traditions and extension identities.
   - Add manifest import/export examples for alternate suits, ranks, courts, and
     numbering systems.
   - Cover configurable and freeform spread layouts with stable snapshots.

3. **Reading lifecycle**
   - Keep `revise` and `append_follow_up` append-aware and revision-safe.
   - Preserve draw provenance, deck snapshots, and placement labels across edits.
   - Leave person/client records, search, encryption, backups, and UI to Oracle
     Studio.

## Deferred features

Deck-local camera recognition, multi-card layout detection, AI interpretation,
online accounts, synchronization, subscriptions, and licensed commercial deck
packs remain post-MVP. Recognition must stay on-device and user-confirmed; AI
must receive recorded placements and disclose any memory it uses.

## Integration boundary

Sibylla remains usable without Astraeus, Oracle Studio, Magnolia, a database, a
UI, or an AI provider. Oracle Studio may store Sibylla artifacts, associate
them with people and sessions, encrypt them, and maintain application-owned
deck-pack metadata. It must not alter Sibylla's canonical content IDs.
