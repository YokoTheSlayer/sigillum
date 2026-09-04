# Contributor instructions for coding agents

## Scope

These instructions apply to the entire repository.

## Architectural rules

- Keep workflow policy in `sigillum-core`; interface crates must remain thin adapters.
- Maintain one run lifecycle, one contract format, one finding format, one repair loop, and one verdict writer.
- Treat OpenSpec planning artifacts as the editable source of truth; do not create parallel editable proposal, spec, design, or task documents.
- Model scout, architect, implementer, verifier, and judge as policy-defined roles rather than fixed model names.
- Deterministic checks have authority over model output.
- Do not give planning or review roles write or unrestricted shell capabilities.
- Do not add Pactum as a runtime dependency or copy its source. Reimplement selected publicly documented concepts clean-room.
- Do not treat OpenSpec prompt guidance as an enforceable capability boundary; enforcement belongs to the core.
- Preserve attribution for any code derived from Signum.

## Required checks

Run these before submitting changes:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Update architecture decisions and schemas when changing public behavior or artifact formats.
