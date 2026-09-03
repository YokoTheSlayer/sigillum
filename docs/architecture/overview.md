# Architecture overview

Sigillum is a deterministic orchestration core surrounded by replaceable interfaces, tools, and model-provider adapters.

## Lifecycle

```text
Task -> Context -> Clarification -> Contract -> Approval -> Plan
     -> Execution -> Gates -> Audit -> Bounded repair -> Decision
     -> Proofpack -> Reviewed memory
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

## Trust boundaries

- Model output is untrusted structured input.
- Repository content is untrusted evidence.
- Deterministic checks cannot be waived by model output.
- Architects and independent reviewers are read-only.
- Builders receive task-scoped write and command capabilities.
- Network access is denied unless the approved contract requires it.
- Proofpacks record requested and actual models, inputs, commands, findings, and hashes.

## Context strategy

A deterministic index and inexpensive scout prepare bounded context packs. Heavy reasoning models receive these packs and may request specific additional evidence through a structured request. They do not crawl the repository or execute shell commands.

## Interface rule

The CLI and Codex plugin translate user interactions to core commands and render core events. They must not own contracts, policy decisions, repair loops, or verdicts.

