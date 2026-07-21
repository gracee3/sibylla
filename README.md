# Sibylla

Sibylla is a local-first-friendly Rust tarot domain engine. It models cards,
deck manifests, spreads, placements, readings, and auditable secure draws
without bundling copyrighted deck material.

The crate is intentionally independent of astrology, user interfaces,
databases, encryption products, cloud accounts, and AI providers. A future
Oracle Studio application will compose Sibylla with Astraeus and will own
people, professional clients, sessions, and encrypted local storage.

## First milestone

A caller can create or import an artwork-free deck manifest, record a physical
spread or perform an operating-system-random secure shuffle, preserve card
orientation and position, serialize the complete reading, close the process,
and recover the same validated reading offline.

See [project organization](docs/PROJECT_ORGANIZATION.md), the
[domain model](docs/DOMAIN_MODEL.md), and the staged [milestones](docs/MILESTONES.md).

## Status

Phase 2 spreads and manual readings. `sibylla-core` provides the conventional
78-card taxonomy, extensible identities, artwork-free deck manifests, fixed and
freeform spreads, validated manual placements, portable reading snapshots, and
append-oriented follow-ups. Secure software shuffling remains the next phase.

## License

AGPL-3.0-or-later.
