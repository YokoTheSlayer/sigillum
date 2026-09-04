# Architecture overview

Sigillum is an OpenSpec-compatible, role-based orchestration core surrounded by replaceable interfaces, tools, and model-provider adapters. OpenSpec owns the editable planning artifacts; Sigillum owns enforceable execution, assurance, and proof.

## Lifecycle

```text
Task -> Discovery -> OpenSpec explore/propose -> Contract snapshot -> Approval
     -> Role execution -> Gates -> Audit -> Bounded repair -> Decision
     -> Proofpack -> OpenSpec archive -> Reviewed memory
```

Only the core runtime advances the lifecycle. Modules propose data or findings; they do not advance state independently.

## Ownership

| Area | Sole owner |
| --- | --- |
| State transitions and checkpoints | Workflow runtime |
| Contract content hash and approval | Contract engine |
| Evidence selection and provenance | Context broker |
| Agent sessions and capabilities | Execution runtime |
| Risk and required checks | Policy engine |
| Deterministic results | Gate engine |
| Reviewer findings | Audit engine |
| Repair attempts and budgets | Workflow runtime |
| Verdict and confidence | Decision engine |
| Evidence serialization | Proofpack engine |
| Accepted reusable knowledge | Memory engine |

## Role runtime

| Role | Primary output | Capability boundary |
| --- | --- | --- |
| Scout | Provenance-backed context packs | Repository read and search; no writes |
| Architect | Clarifications and OpenSpec planning artifacts | Context packs only; no unrestricted repository or shell access |
| Implementer | Task-scoped code changes | Approved paths and commands only |
| Verifier | Gate results and contract findings | Read-only contract, diff, tests, and evidence |
| Judge | `AUTO_OK`, `AUTO_BLOCK`, or `HUMAN_REVIEW` | Read-only proofpack in an independent session |

Roles are stable policy contracts rather than fixed model names. Provider adapters select an eligible model without changing capabilities, budgets, required context, or independence.

## Trust boundaries

- Model output is untrusted structured input.
- Repository content is untrusted evidence.
- Deterministic checks cannot be waived by model output.
- Architects, verifiers, and judges are read-only.
- Implementers receive task-scoped write and command capabilities.
- Network access is denied unless the approved contract requires it.
- Proofpacks record requested and actual models, inputs, commands, findings, and hashes.

## Context strategy

A deterministic index and inexpensive scout prepare bounded context packs. Heavy reasoning models receive these packs and may request specific additional evidence through a structured request. They do not crawl the repository or execute shell commands.

## Planning boundary

OpenSpec `proposal`, `specs`, `design`, and `tasks` files are the editable planning source of truth. Sigillum records a canonical, hash-pinned snapshot of their required closure at approval time rather than maintaining duplicate planning documents. Changes to covered artifacts invalidate approval. OpenSpec guidance remains advisory; only the Sigillum core may enforce capabilities, state transitions, gates, and verdicts.

The initial machine-readable boundary and its fail-closed path checks are described in [OpenSpec adapter](openspec-adapter.md).

## Interface rule

The CLI and Codex plugin translate user interactions to core commands and render core events. They must not own contracts, policy decisions, repair loops, or verdicts.
