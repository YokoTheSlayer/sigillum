# Sigillum Roadmap

> Contract-first orchestration for AI coding agents with risk-adaptive execution, independent review, and verifiable proof.

Sigillum combines the strongest ideas of Signum and Pactum in one product without running two overlapping orchestration engines.

## Target workflow

```text
TASK -> CONTEXT -> CLARIFY -> CONTRACT -> APPROVE -> PLAN
     -> EXECUTE -> GATE -> AUDIT -> FIX -> DECIDE -> PROOFPACK -> MEMORY
```

The same core must serve both interfaces:

- Codex plugin: `/sigillum`
- Standalone CLI: `sigillum run "<task>"`

## Architecture boundaries

Every responsibility has exactly one owner.

| Responsibility | Owner |
| --- | --- |
| Run lifecycle and state machine | Core runtime |
| Contract, content hash, approval | Contract engine |
| Repository discovery and context packs | Context broker |
| Agent execution and model routing | ACP/provider adapters |
| Write scope and command restrictions | Execution guard |
| Risk classification and audit depth | Policy engine |
| Deterministic checks and holdouts | Gate engine |
| Specialist reviews and finding normalization | Audit engine |
| Bounded repair loop | Core runtime |
| Verdict and confidence | Decision engine |
| Evidence bundle | Proofpack engine |
| Reviewed reusable knowledge | Memory engine |
| CLI and Codex UX | Thin adapters over the same core |

There will be one contract, one state tree, one finding schema, one fix loop, and one final verdict.

## Engineering principles

1. One core, multiple interfaces.
2. Approved contracts are immutable; changes require re-approval.
3. Heavy models reason over curated evidence instead of crawling the repository.
4. Context packs include provenance, hashes, paths, line ranges, and critical raw excerpts.
5. Deterministic failures cannot be overridden by an LLM.
6. Scope, time, token, context, and retry budgets are explicit.
7. Every verdict is traceable to recorded evidence.
8. Providers and agent protocols are adapters, not workflow logic.
9. Accepted memory is reviewed, scoped, and invalidated when its source becomes stale.
10. Sigillum does not claim to be a security sandbox.

## Milestone 0 вЂ” Foundation

**Target:** v0.0.1

- [ ] Add Apache-2.0 for original Sigillum code.
- [ ] Preserve MIT attribution for files derived from Signum.
- [ ] Confirm Pactum licensing or use a clean-room implementation of its ideas.
- [ ] Add README, CONTRIBUTING, SECURITY, and ADR template.
- [ ] Select implementation language and package layout.
- [ ] Define versioned artifact schemas and compatibility policy.
- [ ] Configure formatting, linting, unit tests, and CI.
- [ ] Record architecture ownership boundaries as an ADR.

**Exit:** licensed skeleton, passing CI, no unlicensed Pactum source.

## Milestone 1 вЂ” Minimal vertical slice

**Target:** v0.1.0

- [ ] Implement `sigillum init` and `sigillum run`.
- [ ] Store runs under `.sigillum/runs/<run-id>/`.
- [ ] Define schemas for task, context, contract, events, findings, and verdict.
- [ ] Generate a contract and require hash-pinned approval.
- [ ] Execute one coding agent through an adapter.
- [ ] Run configured deterministic checks.
- [ ] Produce Markdown and JSON proofpacks.
- [ ] Return stable exit codes for success, failure, and human review.

**Exit:** one small task completes end to end and can be audited without console history.

## Milestone 2 вЂ” Codex plugin

**Target:** v0.2.0

- [ ] Add the Codex plugin manifest and `/sigillum` command.
- [ ] Invoke the same core used by the CLI.
- [ ] Present clarification, approval, progress, findings, and verdict conversationally.
- [ ] Support status, resume, and cancel.
- [ ] Add parity tests for CLI and plugin artifacts.

**Exit:** CLI and Codex have equivalent lifecycle semantics.

## Milestone 3 вЂ” Context broker

**Target:** v0.3.0

- [ ] Build a deterministic project index cached by commit and working-tree fingerprint.
- [ ] Index instructions, manifests, dependencies, tests, ADRs, and relevant symbols.
- [ ] Add an inexpensive scout role for ambiguous discovery.
- [ ] Produce bounded, evidence-backed `ContextPack` artifacts.
- [ ] Give the architect context-request capability, not unrestricted filesystem access.
- [ ] Support progressive disclosure through bounded delta packs.
- [ ] Gate context quality by provenance, freshness, coverage, and budget.

**Exit:** the architect creates a defensible contract and plan without crawling the project.

## Milestone 4 вЂ” Multi-model orchestration

**Target:** v0.4.0

| Stage | Role |
| --- | --- |
| Discovery | Fast scout model |
| Clarification, contract, architecture | Deep reasoning model (for example GPT Sol) |
| Implementation and routine repair | Fast coding model (for example GPT Luna) |
| Tests and policy checks | Deterministic local tools |
| High-risk review | Independent deep reviewer or reviewer panel |

- [ ] Configure models by role, stage, risk, and task.
- [ ] Record requested and actual model, provider, effort, tokens, and latency.
- [ ] Enforce per-role read/write/shell/network capabilities.
- [ ] Add explicit fallback policies without silently reducing trust.
- [ ] Keep implementer and independent reviewer sessions isolated.
- [ ] Enforce per-stage budgets.

**Exit:** a heavy architect can direct a cheaper builder using curated context and an approved plan.

## Milestone 5 вЂ” Guardrails and gates

**Target:** v0.5.0

- [ ] Enforce allowed and forbidden paths at the agent write boundary.
- [ ] Detect indirect or shell-based out-of-scope changes with snapshots.
- [ ] Enforce command and network policy.
- [ ] Run format, lint, typecheck, tests, dependency, policy, and secret checks.
- [ ] Compare results against a captured baseline.
- [ ] Add holdout tests hidden from the builder.
- [ ] Emit evidence-backed findings with stable IDs.

**Exit:** out-of-scope changes and newly introduced deterministic failures block the run.

## Milestone 6 вЂ” Risk-adaptive audit and repair

**Target:** v0.6.0

- [ ] Classify risk from paths, dependencies, security boundaries, migrations, and contract metadata.
- [ ] Select standard, elevated, or critical audit depth.
- [ ] Add correctness, security, performance, and maintainability reviewers.
- [ ] Normalize and deduplicate all findings.
- [ ] Require evidence for blocking findings.
- [ ] Implement one centrally owned bounded repair loop.
- [ ] Re-run impacted gates after every repair.
- [ ] Escalate unresolved uncertainty or reviewer disagreement to a human.

**Exit:** audit cost scales with risk and repairs cannot loop indefinitely.

## Milestone 7 вЂ” Decision and proofpack

**Target:** v0.7.0

- [ ] Implement `AUTO_OK`, `AUTO_BLOCK`, and `HUMAN_REVIEW`.
- [ ] Calculate confidence from evidence completeness, gates, agreement, and uncertainty.
- [ ] Include contract hash, context hashes, diff, commands, results, findings, model ledger, budgets, and rationale.
- [ ] Redact secrets and configurable sensitive paths.
- [ ] Add CI annotations and pull-request summaries.
- [ ] Protect proofpack integrity with content hashes.

**Exit:** a reviewer can independently understand and verify every verdict.

## Milestone 8 вЂ” Recovery, memory, and hardening

**Target:** v0.8.0

- [ ] Add checkpoints and resume.
- [ ] Make state transitions idempotent.
- [ ] Persist only reviewed and accepted memory.
- [ ] Scope memory by repository, branch, component, and validity window.
- [ ] Detect stale memory against changed evidence.
- [ ] Add artifact migrations and backward-compatible readers.
- [ ] Test cancellation, crashes, provider failures, retries, and budget exhaustion.
- [ ] Publish signed reproducible CLI and plugin packages.

**Exit:** interrupted runs recover without duplicated effects and memory remains traceable.

## Milestone 9 вЂ” Stable release

**Target:** v1.0.0

- [ ] Stabilize CLI, plugin command, configuration, schemas, and exit codes.
- [ ] Publish the threat model and security limitations.
- [ ] Document supported providers and capability guarantees.
- [ ] Add migration guides from Signum and Pactum-style workflows.
- [ ] Benchmark cost, latency, context size, audit depth, and defect detection.
- [ ] Complete an external security and architecture review.
- [ ] Define deprecation and support policy.

## Proposed layout

```text
sigillum/
в”њв”Ђв”Ђ cmd/
в”њв”Ђв”Ђ core/
в”‚   в”њв”Ђв”Ђ workflow/
в”‚   в”њв”Ђв”Ђ contract/
в”‚   в”њв”Ђв”Ђ context/
в”‚   в”њв”Ђв”Ђ execution/
в”‚   в”њв”Ђв”Ђ policy/
в”‚   в”њв”Ђв”Ђ audit/
в”‚   в”њв”Ђв”Ђ decision/
в”‚   в”њв”Ђв”Ђ proofpack/
в”‚   в””в”Ђв”Ђ memory/
в”њв”Ђв”Ђ adapters/
в”‚   в”њв”Ђв”Ђ codex/
в”‚   в”њв”Ђв”Ђ acp/
в”‚   в””в”Ђв”Ђ providers/
в”њв”Ђв”Ђ schemas/
в”њв”Ђв”Ђ tests/
в”‚   в”њв”Ђв”Ђ unit/
в”‚   в”њв”Ђв”Ђ integration/
в”‚   в”њв”Ђв”Ђ fixtures/
в”‚   в””в”Ђв”Ђ holdouts/
в”њв”Ђв”Ђ docs/
в”‚   в”њв”Ђв”Ђ adr/
в”‚   в”њв”Ђв”Ђ architecture/
в”‚   в””в”Ђв”Ђ threat-model/
в””в”Ђв”Ђ .github/workflows/
```

The language-specific layout may change in Milestone 0; ownership boundaries should not.

## Cross-cutting test strategy

Every milestone adds schema compatibility, deterministic state-transition, golden proofpack, provider contract, scope-policy, interruption, role-isolation, and budget-limit tests. End-to-end tests use small fixture repositories and do not require paid model calls by default.

## Non-goals before v1.0

- A general-purpose multi-agent chat framework.
- Unbounded autonomous execution.
- A hosted control plane.
- Supporting every provider.
- Maintaining separate Signum and Pactum engines.
- Copying unlicensed source code.

## Immediate next step

Complete Milestone 0, then build Milestone 1 as a narrow end-to-end slice. Do not start advanced model routing or auditing until the single-agent lifecycle reliably produces a valid proofpack.

