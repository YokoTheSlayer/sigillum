# Contributor instructions for coding agents

## Scope

These instructions apply to the entire repository.

## Architectural rules

- Keep workflow policy in `sigillum-core`; interface crates must remain thin adapters.
- Maintain one run lifecycle, one contract format, one finding format, one repair loop, and one verdict writer.
- Deterministic checks have authority over model output.
- Do not give planning or review roles write or unrestricted shell capabilities.
- Do not copy Pactum source until its license is explicitly compatible. Reimplement documented behavior clean-room.
- Preserve attribution for any code derived from Signum.

## Required checks

Run these before submitting changes:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Update architecture decisions and schemas when changing public behavior or artifact formats.

