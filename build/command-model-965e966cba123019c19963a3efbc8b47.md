---
title: Command Model
label: devflow.developer-guide.design.command-model
---

# Command Model

Devflow uses a two-level command system:

- Primary command: stable top-level verb (`test`, `check`, `ci`).
- Selector: scoped behavior (`test:unit`, `check:pr`, `ci:generate`).

This model keeps UX stable while allowing extension-specific specialization.
