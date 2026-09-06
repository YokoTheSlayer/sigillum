# OpenSpec adapter

The `sigillum-openspec` crate is the only component that translates OpenSpec planning state into a Sigillum contract snapshot. It invokes the configured OpenSpec executable directly, without a shell, and consumes only the documented JSON interfaces.

## Protocol

For `sigillum contract <change>`, the adapter runs these commands in the selected project directory:

```text
openspec --version
openspec status --change <change> --json
openspec validate <change> --type change --strict --json --no-interactive
openspec instructions apply --change <change> --json
```

The adapter requires matching change, schema, project-root, and change-directory identities across the JSON responses. Planning must be complete, strict validation protocol `1.0` must report the selected change as valid, and the apply state must be `ready` or `all_done`. Unknown states and protocol versions fail closed. OpenSpec validation exit code 1 is parsed as a report because that is the documented result for invalid items; other non-zero exits remain command failures.

OpenSpec's `contextFiles` map defines the required artifact closure. Sigillum does not copy or update those files. It reads their exact bytes and passes artifact identifiers, root-relative paths, and content to the core snapshot builder.

The command prints the canonical contract fingerprint followed by every artifact identifier, canonical path, and content digest. It never prints artifact bodies.

## Safety and resource bounds

- Every context file must be an absolute path whose canonical target is inside the canonical OpenSpec change directory.
- The change directory must itself be inside the canonical project root reported by OpenSpec.
- Symlink escapes, directories in place of files, non-UTF-8 relative paths, duplicate paths, and non-canonical core paths are rejected.
- A response may contain at most 1,024 artifact groups/files, each file may be at most 8 MiB, and the closure may be at most 32 MiB.
- JSON input is limited to 4 MiB and 128 nesting levels; duplicate object keys are rejected.

These checks protect the contract boundary but do not make Sigillum a security sandbox. Files can still change between process output and reading; the resulting snapshot fingerprints the bytes that Sigillum actually read.

## Compatibility policy

The adapter supports OpenSpec CLI versions `>=1.12.0,<2.0.0`, the range covered by the agent contract used during implementation. Prerelease and build-qualified versions are rejected until explicitly tested. Missing fields, changed types, malformed JSON, unknown validation protocols, and unknown state values produce an error instead of silently degrading the contract.
