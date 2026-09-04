# Sigillum

OpenSpec-compatible role runtime for AI coding agents with bounded context, controlled execution, independent review, and verifiable proof.

> Status: foundation work (`v0.0.1`). Sigillum is not ready for production use.

## Purpose

Sigillum will connect OpenSpec planning to one traceable, enforceable workflow for coding-agent tasks:

```text
TASK -> DISCOVER -> EXPLORE -> PROPOSE -> CONTRACT -> APPROVE
     -> IMPLEMENT -> VERIFY -> REPAIR -> JUDGE -> PROOFPACK
     -> ARCHIVE -> MEMORY
```

The standalone `sigillum` CLI and the future `/sigillum` Codex plugin will be adapters over the same deterministic core.

OpenSpec owns the editable `proposal`, `specs`, `design`, and `tasks` artifacts. Sigillum turns their required closure into a hash-pinned contract and executes it through five policy-defined roles: scout, architect, implementer, verifier, and judge. Pactum is not a dependency; selected execution concepts may be independently implemented inside the single Sigillum runtime.

## Current capabilities

The foundation and first contract slice establish:

- a Rust workspace split into core, OpenSpec adapter, and CLI crates;
- `sigillum contract <change>` for a fail-closed, canonical snapshot of an OpenSpec artifact closure;
- explicit architecture and artifact compatibility decisions;
- formatting, linting, tests, and CI;
- legal boundaries for OpenSpec integration, Signum-derived work, and clean-room implementation of selected Pactum concepts.

See [ROADMAP.md](ROADMAP.md) for planned milestones and [the architecture overview](docs/architecture/overview.md) for component ownership.

## Build

Install Rust 1.81 or later, then run:

```shell
cargo build --workspace
cargo test --workspace
cargo run --bin sigillum -- --version
cargo run --bin sigillum -- contract <change> --project <path>
```

## Security

Sigillum is an orchestration and evidence system, not a security sandbox. See [SECURITY.md](SECURITY.md) before using it with untrusted repositories or commands.

## License

Original Sigillum code is licensed under Apache-2.0. Third-party and derived work is documented in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
