# SIBYLLA // AGENT HAND-OFF

Sibylla is the independent Rust tarot domain engine in the Oracle family. It
must remain usable without Astraeus, Oracle Studio, Magnolia, a database, a UI,
or an AI provider.

Before changing implementation, read:

1. `README.md`
2. `docs/PROJECT_ORGANIZATION.md`
3. `docs/DOMAIN_MODEL.md`
4. `docs/MILESTONES.md`

Also read `docs/SHUFFLE.md` for randomness changes and `docs/ARTIFACTS.md` for
serialization, compatibility, or content-ID changes.

## Ownership boundary

Sibylla owns canonical card identities, deck manifests, deck-card mappings,
spread definitions, placements, orientation, secure draws, shuffle provenance,
tarot reading documents, validation, and stable serialization.

Oracle Studio owns people and client profiles, sessions spanning multiple
domains, encrypted vault storage, search, backups, deletion workflows, and UI.
It may store a Sibylla reading artifact and associate it with its own entities.

Sibylla must never depend on Astraeus or Magnolia. Similar project conventions
may be shared deliberately, but sibling path dependencies are forbidden.

## Safety and licensing

- Do not commit deck scans, copyrighted artwork or guidebook text, licensed
  fonts, recognition weights, personal readings, client data, or secrets.
- Built-in test decks must contain only original or clearly verified
  public-domain metadata and no artwork.
- Validate all public constructors and all deserialization paths.
- Production shuffling must use operating-system randomness and an unbiased
  Fisher-Yates implementation. Deterministic RNGs are test/injected inputs only.
- Persist algorithm and schema versions; never make a serialized format depend
  on Rust enum layout or collection iteration order.

## Validation and delivery

The ordinary reviewed checks are:

```bash
cargo test --locked
git diff --check
```

Uncached crates may use ordinary network access. Do not add or process artwork,
personal readings, models, datasets, camera input, or other exceptional assets
as part of the default suite. Keep AGPL-3.0-or-later obligations, fixture
provenance, and third-party rights explicit.

Use a focused feature branch. Commit and push the validated change and open a
pull request; incomplete or higher-risk work stays draft. After publication,
send the exact commit, PR, validation, outcome, risks, and next action to the
repository's external coordination record. Do not claim completion until that
remote handoff is verified.
