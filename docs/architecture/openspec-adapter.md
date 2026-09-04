# OpenSpec adapter

The `sigillum-openspec` crate is the only component that translates OpenSpec planning state into a Sigillum contract snapshot. It invokes the configured OpenSpec executable directly, without a shell, and consumes only the documented JSON interfaces.

## Protocol

For `sigillum contract <change>`, the adapter runs these commands in the selected project directory:

```text
openspec --version
openspec status --change <change> --json
openspec instructions apply --change <change> --json
```

The adapter requires matching change, schema, project-root, and change-directory identities across the two JSON responses. Planning must be complete and the apply state must be `ready` or `all_done`. Unknown states fail closed.

OpenSpec's `contextFiles` map defines the required artifact closure. Sigillum does not copy or update those files. It reads their exact bytes and passes artifact identifiers, root-relative paths, and content to the core snapshot builder.

## Safety and resource bounds

- Every context file must be an absolute path whose canonical target is inside the canonical OpenSpec change directory.
- The change directory must itself be inside the canonical project root reported by OpenSpec.
- Symlink escapes, directories in place of files, non-UTF-8 relative paths, duplicate paths, and non-canonical core paths are rejected.
- A response may contain at most 1,024 artifact groups/files, each file may be at most 8 MiB, and the closure may be at most 32 MiB.
- JSON input is limited to 4 MiB and 128 nesting levels; duplicate object keys are rejected.

These checks protect the contract boundary but do not make Sigillum a security sandbox. Files can still change between process output and reading; the resulting snapshot fingerprints the bytes that Sigillum actually read.

## Compatibility policy

This initial adapter records the OpenSpec version string but does not yet declare a supported version range. Compatibility is checked structurally against the documented fields used by the adapter. Missing fields, changed types, malformed JSON, and unknown state values produce an error instead of silently degrading the contract.

OpenSpec validation output will be added before Milestone 1 is complete. Until then, `status` planning completeness and `instructions apply` readiness are necessary inputs, not a replacement for validation.

