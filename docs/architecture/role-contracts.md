# Role contracts

Sigillum roles are stable policy identities, not model names. A provider adapter may choose a different model for a role, but it cannot change the role's capability allowlist or independence requirement.

The built-in v1 contracts define five roles:

| Role | Repository access | Mutable capability | Context boundary | Independent session |
| --- | --- | --- | --- | --- |
| Scout | Brokered read and search | Writes context packs only | Repository discovery | No |
| Architect | None | None | Reads context packs and requests specific evidence | No |
| Implementer | Approved task paths | Task-scoped writes and approved commands | Contract, task, and context pack | No |
| Verifier | Read-only diff | Runs required gates and writes findings | Contract, diff, and evidence | Yes |
| Judge | None | Writes only the verdict | Contract and immutable proofpack | Yes |

Capability checks are deny-by-default: an operation is unavailable unless its stable identifier appears in the role contract. In particular, the architect cannot crawl the repository, and the verifier and judge cannot inherit the implementer's mutable session.

`schemas/role-contract-v1.schema.json` is the public serialization boundary. The Rust definitions are authoritative for built-in defaults; persisted run manifests will record the exact contracts selected for a run rather than infer permissions from a model or prompt.
