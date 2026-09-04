# Artifact versioning policy

Sigillum stores local run artifacts below `.sigillum/runs/<run-id>/`.

## Rules

1. Every machine-readable artifact contains a positive integer `schema_version`.
2. Writers emit only the current schema version.
3. Readers reject unknown future versions with a clear error.
4. Additive optional fields may remain within a schema version.
5. Removing, renaming, retyping, or changing the meaning of a field requires a new schema version.
6. Migrations are explicit, deterministic, idempotent, and preserve the original artifact.
7. Approval hashes cover canonical contract content and its schema version.
8. Evidence references include source path, content hash, and collection time or repository state.
9. Secrets and configured sensitive paths are redacted before persistence.
10. Stable releases document the artifact versions they can read and write.

Schemas will live in `schemas/` beginning with Milestone 1. Golden fixtures and backward-compatibility tests are required for every supported version.

## Contract snapshot v1

`schemas/contract-snapshot-v1.schema.json` defines the public representation. A planning adapter resolves the required OpenSpec artifact closure and supplies exact file bytes to `sigillum-core`. The core:

1. validates change, schema, artifact identifiers, and canonical relative paths;
2. rejects duplicate paths or duplicate artifact identities;
3. calculates a lowercase SHA-256 digest for each exact file body;
4. sorts artifacts by artifact identifier and then relative path;
5. length-prefixes every variable field and hashes the canonical binary representation.

The canonical representation starts with `sigillum-contract-snapshot\0`, followed by the big-endian schema version, length-prefixed change and OpenSpec schema identifiers, artifact count, and length-prefixed identity, path, and content digest fields for each sorted artifact. The snapshot fingerprint does not depend on adapter discovery order. File content, identity, path, OpenSpec schema, or change identity modifications produce a different fingerprint and invalidate approval.

OpenSpec readiness and dependency resolution remain planning-adapter responsibilities. A core snapshot proves exactly what was approved; it does not claim that an incomplete OpenSpec change is ready.
