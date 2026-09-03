# Contributing

Sigillum is in its foundation phase. Small, reviewable changes that preserve the architecture boundaries in `AGENTS.md` are preferred.

## Development setup

1. Install the Rust toolchain declared in `rust-toolchain.toml`.
2. Create a focused branch from `main`.
3. Make the smallest coherent change.
4. Run the required checks.

```shell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Pull requests

- Explain the user-visible outcome and architectural impact.
- Add or update tests for changed behavior.
- Update an ADR for decisions that are difficult to reverse.
- Update schema documentation when artifact formats change.
- Do not include secrets, generated run artifacts, or model transcripts.

## Licensing

By submitting a contribution, you agree that it may be distributed under Apache-2.0. Identify any copied or adapted third-party material and its license in the pull request.

Pactum source must not be copied while its licensing remains unresolved. Compatible behavior may be independently implemented from public behavior and documentation, with clean-room notes recorded in an ADR or pull request.

