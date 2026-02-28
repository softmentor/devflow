---
title: User Guide
label: devflow.user-guide.index
---

# User Guide

## Who This Is For

This guide is for engineers and teams who want a **consistent developer workflow across local machines and CI**, without every repository inventing its own scripts and check logic.

Typical users:

- backend teams (Rust, Node, polyglot repos)
- frontend teams that need repeatable quality gates
- platform teams standardizing CI workflows across projects

## Mission and Objectives

Devflow exists to make workflow behavior predictable and shareable.

Objectives:

- one canonical command surface across stacks
- profile-driven quality gates (`targets.*`) instead of ad hoc scripts
- reproducible local and CI behavior
- extension model that lets projects map canonical commands to stack tooling

## Positioning

Devflow does not replace `Makefile` or `justfile`. It adds a consistent command contract and profile policy layer across repositories, while Make/Just can continue to execute stack-specific internals.

## Why This Matters

Without a shared workflow contract, local and CI behavior drifts. Teams duplicate shell scripts, command names vary by project, and debugging failed CI becomes expensive.

Devflow turns this into a config-defined contract that can be executed and validated consistently.

## Start Here

1. [Problem and Workflow Model](#devflow.user-guide.workflow-model)
2. [Core Principles and Features](#devflow.user-guide.features)
3. [Installation](#devflow.user-guide.installation)
4. [Getting Started](#devflow.user-guide.getting-started)
5. [Configuration](#devflow.user-guide.configuration)
6. [New Stacks (Kotlin Example)](#devflow.user-guide.new-stacks)
