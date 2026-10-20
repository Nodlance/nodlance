# Contributing to Nodlance

Thank you for considering a contribution. Nodlance is pre-alpha, so design discussion and small, well-tested changes are especially useful.

## Before starting

1. Search existing issues and pull requests before opening a duplicate.
2. For broad or breaking changes, open an issue first and describe the use case, alternatives, and security implications.
3. For architectural changes, add or update an Architecture Decision Record under `docs/ADR/`.
4. Keep pull requests focused. Avoid combining unrelated refactors with feature changes.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

If Cargo creates or updates `Cargo.lock`, include it in the pull request for this application workspace. Do not commit build artifacts or local secrets.

## Engineering requirements

- Prefer safe Rust; `unsafe` is forbidden by workspace lint unless an explicitly reviewed ADR explains an exceptional need.
- Validate all untrusted input at trust boundaries.
- Never add arbitrary shell execution endpoints or embed credentials in source/config examples.
- Use typed errors and structured logs; redact credentials, tokens, SQL secrets, and sensitive payloads.
- Side-effecting operations must define authorization, preconditions, timeout, idempotency/retry behavior, audit records, and recovery behavior.
- Engine/provider support requires tests against the relevant version matrix. Do not infer support from compilation alone.
- Do not implement automatic failover based only on heartbeat timeouts.

## Pull request checklist

- [ ] The change has a clear problem statement and bounded scope.
- [ ] Tests cover normal and failure paths where applicable.
- [ ] Formatting, Clippy, and tests pass locally or CI explains the failure.
- [ ] Public API/schema changes are documented.
- [ ] Security and backward-compatibility implications are described.
- [ ] Documentation and relevant ADRs are updated.

## Conduct

Participation is subject to the project's [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
