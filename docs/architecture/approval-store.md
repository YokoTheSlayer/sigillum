# Approval store

Sigillum approval is an explicit acknowledgement of one freshly resolved contract fingerprint. The CLI never infers approval from planning completeness, validation success, or a previous run.

## Flow

1. `sigillum contract <change>` resolves and strictly validates the OpenSpec change, then prints the canonical fingerprint and approval status.
2. A human reviews the contract output.
3. `sigillum approve <change> <fingerprint>` resolves the change again and accepts the command only when the supplied fingerprint exactly matches the fresh snapshot.
4. Sigillum writes a canonical record below `.sigillum/contracts/<change>/approval-<fingerprint>.json`.
5. Later `contract` calls recompute the fingerprint. An exact record is `valid`; older records with no exact match produce `contract_changed`; malformed content at the exact filename produces `approval_record_corrupt`.

Approval records are append-only and idempotent. Re-approving an unchanged contract returns the existing record. Approving a changed contract creates a second record and preserves the previous decision as local history.

## Boundaries

The store contains only the change identifier and contract fingerprint. It does not duplicate OpenSpec planning content. Records intentionally omit timestamps and actor identity until Sigillum has a trusted identity/evidence model; inventing either from local environment data would weaken audit meaning.

State directories and existing records may not be symlinks, and their canonical paths must remain below the OpenSpec root. These checks reduce accidental path escape but do not make the local filesystem a security boundary. A hostile process with write access can still alter or delete local approval history; corrupted exact records fail closed.
