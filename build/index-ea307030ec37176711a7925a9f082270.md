---
title: Devflow - Deterministic Developer Workflows
label: devflow.index
site:
  hide_outline: true
  hide_toc: true
  hide_title_block: true
---

+++ { "kind": "split-image" }

Deterministic Workflows

# One Command Surface
# Local and CI Parity

Devflow provides stable commands and profile-driven orchestration for building, testing, and validating software across stacks.

```{image} assets/images/devflow-landing.svg
:class: only-light
```
```{image} assets/images/devflow-landing.svg
:class: only-dark
```

{button}`Get Started <#devflow.user-guide.getting-started>`

+++ { "kind": "justified" }

## What Devflow Solves

- Consistent command contract (`setup`, `fmt`, `lint`, `build`, `test`, `check`, `release`, `ci`).
- Selector-based precision (`test:unit`, `check:pr`, `ci:generate`).
- Config-first policy (`targets.*`) with deterministic CI generation.

## Who It Is For

- Teams maintaining one or many repositories with inconsistent local/CI workflows.
- Engineers who want predictable quality gates without custom per-repo script sprawl.
- Contributors extending workflow behavior through explicit extension capabilities.

## Mission and Objectives

Mission: make software delivery workflows consistent, repeatable, and easy to reason about.

Objectives:

- define one canonical command surface across stacks
- keep local and CI behavior aligned from the same config contract
- make workflow policy explicit and versionable (`devflow.toml`)
- enable safe extensibility through capability-checked extensions

## Documentation Paths

- [User Guide](#devflow.user-guide.index): install, configure, and run Devflow.
- [Developer Guide](#devflow.developer-guide.index): architecture, extension model, and contribution workflow.
