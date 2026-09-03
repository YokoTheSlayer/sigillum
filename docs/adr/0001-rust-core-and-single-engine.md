# ADR 0001: Rust core and a single workflow engine

- Status: Accepted
- Date: 2026-09-04
- Owners: Sigillum maintainers

## Context

Sigillum combines a standalone orchestrator inspired by Pactum with the policy, audit, and proof-oriented workflow of Signum. Running both original orchestration flows would duplicate contracts, state, repair loops, and verdicts. The product also needs a distributable CLI and a thin Codex plugin over identical semantics.

## Decision

Sigillum will use one deterministic Rust core. Interfaces and model providers are adapters around that core.

The core exclusively owns:

- run state transitions;
- contract hashing and approval validity;
- execution capability enforcement;
- the bounded repair loop;
- final decisions and artifact generation.

Policy and audit modules contribute inputs to the lifecycle but do not create parallel lifecycles. The CLI and Codex plugin may render different user experiences but may not reinterpret state transitions.

Pactum-inspired behavior will be implemented clean-room until compatible licensing is confirmed. Signum-derived code must retain MIT attribution.

## Consequences

- Rust's type system can encode valid state transitions and capability boundaries.
- A single binary can support local and CI use.
- Plugin work requires a stable adapter protocol rather than embedding orchestration in prompts.
- Contributors need a Rust toolchain.
- Direct source reuse from Pactum is out of scope without explicit permission.

## Alternatives considered

- **Go core:** mature for standalone tooling and close to Pactum, but diverges from Signum's proposed `signum-core` direction and makes clean-room separation less obvious.
- **Prompt and script orchestration:** quick to modify, but weak for versioned state, recovery, and invariant enforcement.
- **Two engines behind one command:** rejected because state and authority would be ambiguous.

## Validation

- Core crates must not depend on CLI or Codex adapter crates.
- Parity tests will run identical fixtures through every interface.
- Architecture tests will ensure one verdict writer and one repair-loop owner.

