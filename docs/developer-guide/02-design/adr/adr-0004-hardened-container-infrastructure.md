---
title: ADR-0004 Hardened Container Infrastructure
label: devflow.developer-guide.design.adr-0004
date: 2026-03-01
status: accepted
---

# ADR-0004: Hardened Container Infrastructure

## Context

Devflow's container execution environment was initially functional but lacked professional-grade security hardening and modern build optimizations. To ensure "Devflow-certified" images are secure for enterprise use and high-velocity CI/CD, we need to move beyond basic Dockerfiles and adopt 2025 best practices.

## Decision

We have decided to re-architect our container infrastructure around four pillars:

1. **Docker Bake (HCL)**: Move from raw shell scripts to `docker-bake.hcl`. This enables:
   - Declarative multi-platform builds (`amd64`, `arm64`).
   - Advanced compression (`zstd`) for faster image distribution.
   - Cleaner CI automation.
2. **Security Hardening**:
   - **Non-root user**: All images will run as `dwfuser` (UID 1001) instead of `root`.
   - **Init Process**: Integrating `tini` as the container entrypoint to handle signal propagation and zombie process reaping (crucial for long-running CI tasks).
3. **BuildKit Optimizations**:
   - Using `--mount=type=cache` for persistent `apt` and `cargo` registries, significantly reducing rebuild times by avoiding redundant network requests.
4. **Automated Vulnerability Scanning**:
   - Integrating **Trivy** directly into the CI build pipeline (`prep` job) with a policy to fail builds containing `CRITICAL` or `HIGH` vulnerabilities.

## Consequences

### Positive
- **Security**: Reduced attack surface by running as a non-privileged user and automated CVE guarding.
- **Speed**: `zstd` compression and BuildKit cache mounts significantly reduce image pull times and build durations.
- **Portability**: Verified architecture-agnostic builds across Apple Silicon and GitHub Actions.

### Negative
- **Complexity**: Docker Bake adds another configuration file and dependency to the project.
- **Permissions**: Moving to a non-root user requires careful `chown` management of workspace mounts, currently handled via `sudo chmod` in CI templates as a workaround for GHA's default root behavior.
