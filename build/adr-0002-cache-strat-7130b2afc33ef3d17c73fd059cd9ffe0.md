---
title: ADR-0002 Cache Strategy Across Host, Container, and CI
label: devflow.adr.0002.cache-strategy
---

# ADR-0002 Cache Strategy Across Host, Container, and CI

## Status
Accepted

## Decision

Adopt profile-aware cache partitioning where cache keys and directories are isolated by runtime profile (`host` vs `container`) and environment dimensions (`os`, `arch`, toolchain lock).

## Rationale

- Prevent artifact contamination across heterogeneous runtimes.
- Keep cache as optimization only; correctness must not rely on hits.
- Align local and CI cache behavior with explicit boundaries.

## Consequences

- More cache directories/keys to manage.
- Higher storage usage, but significantly lower risk of false positives/negatives in builds and tests.
