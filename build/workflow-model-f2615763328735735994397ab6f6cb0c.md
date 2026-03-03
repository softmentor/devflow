---
title: Problem and Workflow Model
label: devflow.user-guide.workflow-model
---

# Problem and Workflow Model

## Typical Workflow Today (Without Devflow)

```mermaid
flowchart LR
    local1["Local scripts: npm run lint"]
    local2["Local scripts: cargo test"]
    local3["Custom make targets"]
    ci1["CI YAML job A"]
    ci2["CI YAML job B"]
    ci3["Repo-specific glue scripts"]

    local1 --> ci1
    local2 --> ci2
    local3 --> ci3
```

Common issues:

- duplicate logic across scripts, Makefiles, and CI YAML
- inconsistent command names across repositories
- local success but CI failure due to workflow drift

## Workflow With Devflow

```mermaid
flowchart LR
    cfg["devflow.toml"]
    cli["dwf command surface"]
    ext["stack extensions"]
    local["local execution"]
    ci["generated CI workflow"]

    cfg --> cli
    cli --> ext
    ext --> local
    cfg --> ci
    cli --> ci
```

Benefits:

- same commands locally and in CI
- DRY policy definition in `targets.*`
- reproducible workflow generation and validation (`ci:generate`, `ci:check`)
