# Secure shuffle contract

Status: Fisher-Yates algorithm version 1 implemented
Last reviewed: 2026-07-21

## Production boundary

`sibylla_shuffle::shuffle` is the production entrypoint. It constructs an
`OsRandom` source internally and obtains bytes through `getrandom`, which uses
the operating system's cryptographic random-number facility. No deterministic
seed or testing source is selected implicitly.

`shuffle_with_source` is the explicit injection seam for deterministic tests or
caller-controlled replays. Its provenance records `injected`.

## Fisher-Yates v1

1. Copy the manifest's enabled physical deck cards in manifest order.
2. Iterate the active upper bound from the population length down through two.
3. Read an unsigned 64-bit sample in little-endian byte order.
4. Let `zone = floor(2^64 / upper) * upper`; reject samples at or above `zone`.
5. Reduce an accepted sample modulo `upper` and swap it with `upper - 1`.
6. After order is complete, sample each card's orientation independently.

The rejected tail is what prevents modulo bias. These steps and their byte order
are part of algorithm version 1 and may not change without a new version and
pinned fixture.

## Reversals

The manifest stores a rate from 0 through 10,000 basis points. Intermediate
rates use the same unbiased sampler over `[0, 10000)` and mark a card reversed
when the sample is below the configured rate. A zero rate always returns
upright, a 10,000 rate always returns reversed, and neither boundary consumes
reversal entropy. Software shuffles never return unspecified orientation.

## Provenance and seeds

Every result records the algorithm ID/version, source category, exact
draw-manifest ID, enabled-card count, reversal policy, and caller-supplied UTC
timestamp. An optional SHA-256 seed commitment can be recorded, but raw seeds
are neither requested nor persisted by the production API. Encrypted seed
storage, if desired, belongs to the application vault.

The checked-in `fisher-yates-v1.json` fixture pins order and orientation for a
public non-secret test seed and deterministic SHA-256 counter source.
