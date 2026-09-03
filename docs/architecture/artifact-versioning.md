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

