# ADR 0002: OpenSpec-compatible planning and a role-based runtime

- Status: Proposed
- Date: 2026-09-04
- Owners: Sigillum maintainers

## Context

OpenSpec provides a useful planning workflow around `proposal`, `specs`, `design`, and `tasks` artifacts. It deliberately acts as an agreement layer: its agent guidance is advisory and its apply workflow does not enforce model isolation, write scope, approval integrity, deterministic gates, or independent verdicts.

Sigillum needs those runtime guarantees, but should not maintain a second, competing set of planning documents. Pactum demonstrates useful execution concepts, yet using it as a second engine would duplicate lifecycle ownership and its source cannot be incorporated without confirmed compatible licensing.

## Decision

Sigillum will be an OpenSpec-compatible role runtime with Signum-grade assurance.

OpenSpec artifacts are the human-readable planning source of truth. Sigillum will initially integrate through the OpenSpec CLI JSON interface and may add a native compatible reader later. It will not copy `proposal`, `specs`, `design`, or `tasks` into parallel editable Sigillum documents.

Before approval, planning artifacts remain fluid. Approval creates a canonical contract snapshot covering the selected OpenSpec artifact closure, schema version, and content hashes. Any covered change invalidates approval.

The execution runtime will use five logical roles:

| Role | Responsibility | Default capability boundary |
| --- | --- | --- |
| Scout | Discover relevant repository evidence | Broad read and search; no writes |
| Architect | Clarify and produce or review the plan | Context packs and planning artifacts; no repository crawl or writes |
| Implementer | Apply approved tasks | Task-scoped reads, writes, and commands |
| Verifier | Compare implementation with contract and run checks | Read-only diff, contract, and evidence access |
| Judge | Produce the final verdict | Read-only contract and proofpack access |

A role is a policy-defined responsibility, not a permanently assigned model. Model routing may select a provider and model by capability, risk, cost, and availability while preserving the role's permissions, context limits, budgets, and independence requirements.

The context broker mediates progressive disclosure. Heavy reasoning roles receive bounded context packs with provenance and may request specific additional evidence; they do not scan the repository directly. Prompt instructions are not treated as enforcement. The Rust core enforces lifecycle, capabilities, approval validity, budgets, gates, repair bounds, and verdict authority.

Pactum will not be a runtime dependency or source-code base. Selected public concepts such as contract approval, model routing, scoped execution, retries, and event journaling may be independently implemented in the Sigillum core.

## Consequences

- OpenSpec can evolve independently while Sigillum owns enforceable execution and assurance.
- Users keep one editable planning artifact set and one authoritative runtime state.
- The initial integration requires an installed compatible OpenSpec CLI; a native reader can remove that requirement later.
- Role contracts and hand-off artifacts become public, versioned Sigillum interfaces.
- Provider adapters cannot broaden a role's capabilities or reinterpret workflow state.
- OpenSpec validation improves artifact quality but cannot replace Sigillum gates or proof.

## Alternatives considered

- **Embed or fork Pactum:** rejected because it would introduce a second orchestration model, duplication, and unresolved source-licensing risk.
- **Use OpenSpec apply as the runtime:** rejected because its workflow relies on agent compliance and does not provide the required enforcement or evidence model.
- **Maintain Sigillum-specific planning documents:** rejected because they would duplicate OpenSpec artifacts and create ambiguous sources of truth.
- **Bind each role to one named model:** rejected because it weakens portability, fallback handling, and cost-aware routing.

## Validation

- Compatibility fixtures must map OpenSpec artifacts to one canonical contract snapshot without creating editable duplicates.
- Mutating any approved input artifact must invalidate approval.
- Capability tests must prove that each role cannot exceed its configured read, write, shell, network, and context boundaries.
- Architect and judge fixtures must complete without unrestricted repository access.
- The verifier and judge must not share the implementer's mutable session.
- Deterministic gate failures must remain authoritative over every model role.
