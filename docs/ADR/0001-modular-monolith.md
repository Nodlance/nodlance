# ADR 0001: Begin with a modular monolith

- **Status:** Accepted for initial scaffold
- **Date:** 2026-10-20

## Context

The project is early-stage and intended to attract community contributions. Multiple independently deployed services would add operational burden before scale and ownership boundaries are understood.

## Decision

Use a Rust workspace with a modular control-plane application and explicit boundaries between domain, application, API, workers, agents, engine adapters, and deployment providers. Introduce separately deployed processes only where trust boundaries, fault isolation, or scaling require them.

## Consequences

- Easier local development and end-to-end testing.
- Clear crate interfaces for contributors.
- Some components will be extracted later if measured operational needs justify it.
- Crate boundaries and dependency direction must be enforced in CI and code review.
