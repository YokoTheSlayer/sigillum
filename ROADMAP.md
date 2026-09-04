# Sigillum Roadmap

> OpenSpec-compatible role runtime for AI coding agents with bounded context, controlled execution, independent review, and verifiable proof.

Sigillum combines OpenSpec planning with a deterministic role runtime and Signum-grade assurance. Pactum is not a dependency or a second engine; selected public execution concepts may be independently implemented where they fit the single Sigillum lifecycle.

## Target workflow

```text
TASK -> DISCOVER -> EXPLORE -> PROPOSE -> CONTRACT -> APPROVE
     -> IMPLEMENT -> VERIFY -> REPAIR -> JUDGE -> PROOFPACK
     -> ARCHIVE -> MEMORY
```

OpenSpec owns the editable `proposal`, `specs`, `design`, and `tasks` artifacts. Sigillum turns their required closure into a canonical, hash-pinned contract snapshot and owns every enforceable transition after that boundary.

The same core must serve both interfaces:

- Codex plugin: `/sigillum`
- Standalone CLI: `sigillum run "<task>"`

## Roles

Roles are versioned policy contracts, not fixed model names.

| Role | Responsibility | Default capability boundary |
| --- | --- | --- |
| Scout | Discover relevant repository evidence | Broad read and search; no writes |
| Architect | Clarify and produce or review OpenSpec artifacts | Context packs only; no repository crawl or writes |
| Implementer | Apply approved tasks | Task-scoped reads, writes, and commands |
| Verifier | Compare implementation with the contract and run checks | Read-only diff, contract, tests, and evidence |
| Judge | Produce the final verdict | Read-only contract and proofpack in an independent session |

Provider adapters select eligible models by capability, risk, cost, and availability without changing role permissions, budgets, context limits, or independence requirements.

## Architecture boundaries

Every responsibility has exactly one owner.

| Responsibility | Owner |
| --- | --- |
| Editable planning artifacts | OpenSpec |
| OpenSpec compatibility and artifact import | Planning adapter |
| Run lifecycle and state machine | Core runtime |
| Contract snapshot, content hash, and approval | Contract engine |
| Repository discovery and context packs | Context broker |
| Role definitions and hand-offs | Role engine |
| Agent sessions and model routing | Execution runtime |
| Write scope and command restrictions | Execution guard |
| Risk classification and audit depth | Policy engine |
| Deterministic checks and holdouts | Gate engine |
| Specialist reviews and finding normalization | Audit engine |
| Bounded repair loop | Core runtime |
| Verdict and confidence | Decision engine |
| Evidence bundle | Proofpack engine |
| Reviewed reusable knowledge | Memory engine |
| CLI and Codex UX | Thin adapters over the same core |

There will be one approved contract snapshot, one state tree, one finding schema, one repair loop, and one final verdict. Sigillum will not create editable duplicates of OpenSpec planning artifacts.

## Engineering principles

1. One core, multiple interfaces.
2. OpenSpec artifacts remain fluid until Sigillum approval.
3. Approved contract snapshots are immutable; covered changes require re-approval.
4. Roles define capabilities and responsibilities independently of model names.
5. Heavy models reason over curated evidence instead of crawling the repository.
6. Context packs include provenance, hashes, paths, line ranges, and critical raw excerpts.
7. Deterministic failures cannot be overridden by an LLM.
8. Scope, time, token, context, and retry budgets are explicit.
9. Every verdict is traceable to recorded evidence.
10. Providers, agent protocols, OpenSpec, CLI, and Codex are adapters around core policy.
11. Accepted memory is reviewed, scoped, and invalidated when its source becomes stale.
12. Sigillum does not claim to be a security sandbox.

## Milestone 0 - Foundation

**Target:** v0.0.1

- [x] Add Apache-2.0 for original Sigillum code.
- [x] Preserve MIT attribution for files derived from Signum.
- [x] Exclude unlicensed Pactum source and use clean-room implementation where needed.
- [x] Add README, CONTRIBUTING, SECURITY, and ADR template.
- [x] Select implementation language and package layout.
- [x] Define versioned artifact schemas and compatibility policy.
- [x] Configure formatting, linting, unit tests, and CI.
- [x] Record architecture ownership boundaries as an ADR.

**Exit:** licensed skeleton, passing CI, no unlicensed Pactum source.

## Milestone 1 - OpenSpec-compatible contract

**Target:** v0.1.0

- [ ] Detect a local OpenSpec project and compatible CLI version.
- [ ] Read `status`, `instructions`, and validation results through the OpenSpec JSON interface.
- [x] Resolve the required `proposal`, `specs`, `design`, and `tasks` artifact closure without duplicating editable content.
- [ ] Define versioned schemas for OpenSpec references, contract snapshots, approvals, and invalidation reasons.
- [x] Canonicalize artifact content, paths, schema identity, and hashes.
- [ ] Require explicit hash-pinned approval.
- [ ] Invalidate approval when any covered artifact or schema input changes.
- [ ] Add compatibility fixtures for valid, incomplete, changed, and unsupported OpenSpec projects.
- [x] Document the boundary between advisory OpenSpec guidance and enforceable Sigillum policy.

**Exit:** `sigillum contract <change>` produces one auditable snapshot and detects every covered post-approval change.

## Milestone 2 - Minimal role-runtime slice

**Target:** v0.2.0

- [ ] Implement `sigillum init`, `sigillum run`, `status`, `resume`, and `cancel`.
- [ ] Store runs under `.sigillum/runs/<run-id>/`.
- [ ] Define schemas for roles, hand-offs, events, findings, budgets, and verdicts.
- [ ] Implement all five role identities with minimal initial capability policies.
- [ ] Execute one OpenSpec task through an implementer adapter.
- [ ] Run a deterministic verifier after implementation.
- [ ] Produce minimal Markdown and JSON proofpacks.
- [ ] Return stable exit codes for success, failure, and human review.
- [ ] Test that planner and reviewer roles cannot obtain implementer capabilities.

**Exit:** one small OpenSpec change completes end to end through explicit role boundaries and can be audited without console history.

## Milestone 3 - Context broker and Explore

**Target:** v0.3.0

- [ ] Build a deterministic project index cached by commit and working-tree fingerprint.
- [ ] Index instructions, manifests, dependencies, tests, ADRs, existing OpenSpec artifacts, and relevant symbols.
- [ ] Implement the inexpensive scout role.
- [ ] Produce bounded, provenance-backed `ContextPack` artifacts.
- [ ] Feed curated discovery evidence into OpenSpec Explore and Propose workflows.
- [ ] Give the architect structured context-request capability, not unrestricted filesystem access.
- [ ] Support progressive disclosure through bounded delta packs.
- [ ] Gate context quality by provenance, freshness, coverage, and budget.

**Exit:** the architect can clarify and review a defensible OpenSpec plan without crawling the project.

## Milestone 4 - Codex plugin

**Target:** v0.4.0

- [ ] Add the Codex plugin manifest and `/sigillum` command.
- [ ] Invoke the same core used by the CLI.
- [ ] Bridge OpenSpec Explore, Propose, contract approval, implementation, verification, and verdict conversationally.
- [ ] Present role transitions, context requests, approval invalidation, findings, and budgets.
- [ ] Support status, resume, and cancel.
- [ ] Add parity tests for CLI and plugin artifacts.

**Exit:** CLI and Codex have equivalent lifecycle, contract, role, and verdict semantics.

## Milestone 5 - Multi-model role orchestration

**Target:** v0.5.0

- [ ] Configure models by role, capability, risk, cost, and task.
- [ ] Record requested and actual model, provider, effort, tokens, latency, and fallback path.
- [ ] Enforce per-role read, write, shell, network, and context capabilities.
- [ ] Add explicit fallback policies without silently reducing trust.
- [ ] Keep implementer, verifier, and judge sessions isolated where policy requires it.
- [ ] Enforce per-role and per-stage budgets.
- [ ] Support a deep architect directing a cheaper coding implementer through typed hand-offs.
- [ ] Test provider substitution without changing role semantics.

**Exit:** multiple models cooperate through stable roles while the core preserves capability and trust boundaries.

## Milestone 6 - Guardrails and deterministic gates

**Target:** v0.6.0

- [ ] Enforce allowed and forbidden paths at the agent write boundary.
- [ ] Detect indirect or shell-based out-of-scope changes with snapshots.
- [ ] Enforce command and network policy.
- [ ] Run format, lint, typecheck, tests, dependency, policy, and secret checks.
- [ ] Compare results against a captured baseline.
- [ ] Add holdout tests hidden from the implementer.
- [ ] Emit evidence-backed findings with stable IDs.

**Exit:** out-of-scope changes and newly introduced deterministic failures block the run.

## Milestone 7 - Risk-adaptive audit and repair

**Target:** v0.7.0

- [ ] Classify risk from paths, dependencies, security boundaries, migrations, and contract metadata.
- [ ] Select standard, elevated, or critical audit depth.
- [ ] Add correctness, security, performance, and maintainability verification profiles.
- [ ] Normalize and deduplicate findings from tools and model roles.
- [ ] Require evidence for blocking findings.
- [ ] Implement one centrally owned bounded repair loop.
- [ ] Re-run impacted gates after every repair.
- [ ] Escalate unresolved uncertainty or reviewer disagreement to a human.

**Exit:** audit cost scales with risk and repairs cannot loop indefinitely.

## Milestone 8 - Decision, proof, recovery, and memory

**Target:** v0.8.0

- [ ] Implement judge verdicts: `AUTO_OK`, `AUTO_BLOCK`, and `HUMAN_REVIEW`.
- [ ] Calculate confidence from evidence completeness, gates, agreement, and uncertainty.
- [ ] Include contract and context hashes, diff, commands, results, findings, role/model ledger, budgets, and rationale.
- [ ] Protect proofpack integrity with content hashes and redact secrets.
- [ ] Add checkpoints, idempotent resume, and cancellation.
- [ ] Persist only reviewed and accepted memory with source invalidation.
- [ ] Test crashes, provider failures, retries, budget exhaustion, and stale evidence.
- [ ] Publish signed reproducible CLI and plugin packages.

**Exit:** runs recover without duplicated effects and every verdict remains independently understandable and verifiable.

## Milestone 9 - Stable release

**Target:** v1.0.0

- [ ] Stabilize CLI, plugin commands, role contracts, configuration, schemas, and exit codes.
- [ ] Publish the threat model and security limitations.
- [ ] Document supported OpenSpec versions, providers, and capability guarantees.
- [ ] Add migration guidance from Signum and Pactum-style workflows.
- [ ] Benchmark cost, latency, context size, role hand-offs, audit depth, and defect detection.
- [ ] Complete an external security and architecture review.
- [ ] Define deprecation and support policy.

## Cross-cutting test strategy

Every milestone adds schema compatibility, deterministic state-transition, approval-invalidation, role-capability, role-isolation, golden proofpack, provider contract, scope-policy, interruption, and budget-limit tests. End-to-end tests use small fixture repositories and do not require paid model calls by default.

## Non-goals before v1.0

- A general-purpose multi-agent chat framework.
- Unbounded autonomous execution.
- A hosted control plane.
- Supporting every provider.
- Forking or embedding Pactum.
- Reimplementing the complete OpenSpec CLI.
- Maintaining duplicate editable planning artifacts.
- Treating prompt instructions as security enforcement.

## Immediate next step

Build Milestone 1 as a narrow OpenSpec compatibility and contract-snapshot slice. Do not begin model routing or autonomous implementation until artifact resolution, approval hashing, and invalidation are deterministic and covered by fixtures.
