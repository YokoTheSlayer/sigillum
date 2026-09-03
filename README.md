# Sigillum

Contract-first orchestration for AI coding agents with risk-adaptive execution, independent review, and verifiable proof.

> Status: foundation work (`v0.0.1`). Sigillum is not ready for production use.

## Purpose

Sigillum will provide one traceable workflow for coding-agent tasks:

```text
TASK -> CONTEXT -> CLARIFY -> CONTRACT -> APPROVE -> PLAN
     -> EXECUTE -> GATE -> AUDIT -> FIX -> DECIDE -> PROOFPACK -> MEMORY
```

The standalone `sigillum` CLI and the future `/sigillum` Codex plugin will be adapters over the same deterministic core.

## Current capabilities

Milestone 0 establishes:

- a Rust workspace split into core and CLI crates;
- a minimal `sigillum help` and `sigillum version` executable;
- explicit architecture and artifact compatibility decisions;
- formatting, linting, tests, and CI;
- legal boundaries for Signum-derived work and Pactum-inspired clean-room work.

See [ROADMAP.md](ROADMAP.md) for planned milestones and [the architecture overview](docs/architecture/overview.md) for component ownership.

## Build

Install Rust 1.81 or later, then run:

```shell
cargo build --workspace
cargo test --workspace
cargo run --bin sigillum -- --version
```

## Security

Sigillum is an orchestration and evidence system, not a security sandbox. See [SECURITY.md](SECURITY.md) before using it with untrusted repositories or commands.

## License

Original Sigillum code is licensed under Apache-2.0. Third-party and derived work is documented in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).


