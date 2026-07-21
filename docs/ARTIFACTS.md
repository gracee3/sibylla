# Artifact contract

Status: envelope schema version 1 frozen on 2026-07-21

`sibylla-artifacts` is the stable interchange boundary for independent
applications. It wraps a validated `DeckManifest` or `TarotReading`; it does
not provide storage, encryption, networking, or user identity.

## Envelope

Every artifact is a strict JSON object with this field order in canonical
output:

```json
{
  "schema_version": 1,
  "artifact_type": "deck",
  "payload": {}
}
```

`artifact_type` is `deck` or `reading`. The payload retains its own domain
schema version, which is validated independently. Unknown envelope or payload
fields, duplicate fields, malformed values, and unsupported schema versions
are errors. Typed readers also reject the other artifact type.

## Canonical bytes and content IDs

The canonical form is UTF-8 compact JSON returned by `to_json`. The serializer
fixes envelope and domain field order, includes explicit nulls, and preserves
array order. No trailing newline is part of the canonical bytes.

An artifact content ID is lowercase `sha256:` followed by the SHA-256 digest of
those exact canonical bytes. The pretty representation is for display and is
never hashed directly; parsing it and serializing canonically produces the same
ID. Any envelope or payload change produces a different artifact revision and
content ID.

Content IDs establish integrity and identity, not authenticity, secrecy, or
authorization. A caller that needs those properties must add authenticated
encryption and key management outside Sibylla.

## Compatibility and migration policy

- Envelope version 1 is read and written exactly as documented here.
- Unsupported envelope or nested domain versions fail explicitly. Readers do
  not guess, silently discard fields, or silently upgrade or downgrade data.
- Additive fields require a new schema version because version 1 rejects
  unknown fields.
- A future migration must name its source and target versions, validate both
  sides, preserve the original artifact until replacement is committed, and
  document whether the canonical bytes or meaning change.
- Existing canonical serializers and pinned fixture digests cannot change in a
  compatible patch. An intentional wire change requires a new schema version
  and new golden fixtures.
- Consumers should store the artifact bytes, content ID, and producing Sibylla
  revision together. Cross-repository consumers must pin a Git revision or
  released crate version, never a committed sibling path dependency.

The checked-in deck and reading digest fixtures lock the version 1 canonical
contract. Public-API consumer tests demonstrate export, type dispatch, digest
calculation, and complete offline recovery without depending on Oracle Studio.
