---
title: Core Principles and Features
label: devflow.user-guide.features
---

# Core Principles and Features

## Principles

- Canonical command contract: stable top-level verbs (`setup`, `fmt`, `lint`, `build`, `test`, `check`, `release`, `ci`).
- Selector precision: scoped behavior with `primary:selector` (`test:unit`, `check:pr`).
- Config as policy: target profiles live in `devflow.toml`, not duplicated shell/YAML snippets.
- Reproducibility first: workflow intent is explicit and machine-validated.
- Complement, not replacement: Devflow standardizes command intent and orchestration; your existing Makefile/justfile can still own stack-specific internals.

## Feature Summary

- dynamic target profiles (`pr`, `main`, `staging`, etc.)
- strict config validation (unknown keys fail)
- extension capability checks before execution
- CI workflow generation (`ci:generate`) and topology validation (`ci:check`)
- stack-aware command execution (currently Rust and Node)
- custom stack delegation to `just`/`make` targets for new ecosystems (for example Kotlin)
