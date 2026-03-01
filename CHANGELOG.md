# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-01

### Added
- **Container Execution**: Support for running Devflow tasks inside isolated containers (Docker/Podman).
- **Deterministic Fingerprinting**: Environment identity based on `Dockerfile` and toolchain manifests.
- **Unified Cache Parity**: Consistent volume mounting strategy for `.cargo`, `.npm`, and extension caches across local and CI.
- **GitHub Actions Generator**: Modernized `dwf ci:generate` with Buildx layer caching and native container execution.
- **Subprocess Extensions**: Support for JSON-RPC subprocess extensions within containerized boundaries.
- **Hardened Container Infrastructure**: Universal security hardening across all stacks (Rust, Node, Python, Tauri) using 2025 best practices.
- **Docker Bake Integration**: Centralized, declarative build management with `zstd` compression and multi-platform support.
- **Vulnerability Scanning**: Integrated Trivy automated scanning into the CI pipeline to fail on CRITICAL/HIGH CVEs.
- **Environmental Cleanup**: Added `make teardown` and `dwf prune:*` commands for localized and remote (GHA) cache/run management.

### Changed
- Refactored `executor.rs` for modular container orchestration.
- Simplified image tagging to rely on SHA256 fingerprints rather than explicit version strings.
- Consolidated parallel CI verify jobs into a single sequential runner to reduce restoration overhead.

### Fixed
- QEMU architecture mismatches by dynamically detecting host architecture for CI toolchain downloads.

## [0.1.0] - 2026-02-15

### Added
- Initial release of Devflow.
- Basic Rust and Node.js stack support.
- Multi-stack workspace orchestration.
- Built-in GitHub Actions workflow generation.
