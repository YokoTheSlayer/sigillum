# Security policy

## Supported versions

Sigillum is pre-release software. Only the latest commit on `main` receives security fixes until a stable release policy is published.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private vulnerability reporting feature for this repository. Include affected versions, reproduction steps, impact, and any suggested mitigation.

## Security boundary

Sigillum coordinates agents and checks evidence. It is not an operating-system sandbox.

- Tool capability policy reduces available actions but does not replace process isolation.
- Write-scope checks detect and block policy violations but cannot undo every external side effect.
- Shell and network execution must be isolated by the caller when processing untrusted input.
- Secrets must not be placed in prompts, proofpacks, run logs, or repository configuration.

Security-sensitive releases must document their threat model and known limitations.

