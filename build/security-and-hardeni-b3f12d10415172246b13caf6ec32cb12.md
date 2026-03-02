---
title: Security and Hardening
label: devflow.architecture.security-and-hardening
---

# Security and Hardening Deep Dive

Devflow follows a **"Push-Left"** security philosophy: catching vulnerabilities and misconfigurations as early as possible in the development lifecycle. This document details the specific scanning layers and hardening techniques integrated into the platform.

## 1. Multi-Layer Scanning Strategy

Devflow orchestrates four distinct security scanning layers to ensure comprehensive coverage:

| Layer | Tooling | Focus | Frequency |
| :--- | :--- | :--- | :--- |
| **Code** | GitHub Advanced Security (CodeQL) | Logic Flaws, Logic errors | Every Push/PR |
| **Dependencies** | Trivy (FS mode) | Known CVEs in OSS libraries | Local CI & Every PR |
| **Images/Tars** | Trivy (Image mode) | OS packages, Base image CVEs | Image Build & Nightly |
| **Infra (IaC)** | Trivy (Config mode) | Dockerfile misconfigurations | Local CI & Release |

### 🔄 Vulnerability Database Sync

A common question is how to ensure the vulnerability database (VDB) is synchronized between local and CI environments.

- **Auto-Update**: Trivy automatically checks for and downloads database updates on every run. This ensures that both local and CI environments are checking against the latest known CVEs from the same upstream sources (GHSA, NVD).
- **CI Persistence**: In GitHub Actions, we use `aquasecurity/setup-trivy` with `cache: true` to persist the VDB across jobs, reducing restoration time.
- **Local Consistency**: To manually trigger a fresh database pull locally, you can run `trivy image --download-db-only`.

### Operational Efficiency (Friction Reduction)
Scanning every image on every PR check can be resource-intensive. Devflow optimizes this by:
- **Scan-on-Change**: Full image/tar scans are only triggered in CI when the `Dockerfile.devflow` or toolchain manifests (`Cargo.lock`, `package-lock.json`) are modified.
- **Nightly Guardrails**: A scheduled cron (`0 0 * * *`) executes a full scan on the `main` branch, ensuring that "Zero-Day" vulnerabilities in stagnant base images are caught within 24 hours.

## 2. Hardened-by-Default Infrastructure

Every container generated or managed by Devflow is hardened using 2025 industry standards:

### Non-Root Execution
All Devflow-certified images (Rust, Node, Python, Tauri) execute as `dwfuser` (UID 1001). This prevents privilege escalation attacks where a compromised build process could gain root access to the host runner or developer machine.

### Process Hygiene (Tini)
We use `tini` as the container `ENTRYPOINT`. It acts as an init process, correctly forwarding signals (SIGTERM, SIGINT) and reaping zombie processes. This adds stability to long-running CI parallel tasks.

### Rootless Workflow Compatibility
The Devflow executor is tested for compatibility with **rootless Podman** and **Docker Rootless Mode**, ensuring developers don't need `sudo` to run their local verification suites.

## 3. Complex Stack Hardening

Devflow's hardening strategy extends to modern, complex development use cases:

### Multi-Stack (e.g., Tauri)
For projects involving multiple language toolchains (e.g., Rust + Node.js), Devflow uses a **Layered Base Stage** approach. 
1.  **Shared Base**: A single Debian-slim stage contains the OS-level dependencies (GTK, WebKit, etc.) and `tini`.
2.  **Toolchain Injection**: Subsequent stages inherit from this base, ensuring that security patches and the init process are consistent across the entire multi-stack environment.
3.  **Unified User**: Both Node and Rust processes operate under the same `dwfuser` (UID 1001), simplifying permission management for shared `/workspace` volumes.

### Subprocess Extensions (e.g., Python-ext)
Devflow encourages a "Security-by-Isolation" model for extensions:
- **Trust Gate**: Subprocess extensions are untrusted by default and must explicitly opt in (`trusted = true`) for host negotiation in container profile.
- **Sandbox Boundary**: Command execution still runs inside the hardened container boundary after negotiation.
- **Protocol Security**: Communication between Devflow and the subprocess occurs over standard I/O (JSON-RPC), avoiding the need for network sockets or elevated privileges.
- **Runtime Hardening**: Python-based extensions leverage dedicated `pip` cache mounts and a non-root environment, preventing extension-based supply chain attacks from escalating to the host.

## 4. Least Privilege GITHUB_TOKEN

Devflow's `ci:generate` command explicitly configures the GitHub Actions workflow with a root-level `permissions` block:

```yaml
permissions:
  contents: read
```

This prevents the `GITHUB_TOKEN` from having broad write access by default, protecting your repository from unauthorized modifications during the build phase.

## 5. Setup and Automation

When running `dwf init`, Devflow automatically:
1.  Generates the hardened `Dockerfile.devflow` (correctly versioned to the latest stable toolchains, e.g., Rust 1.93.1).
2.  Configures `docker-bake.hcl` for high-speed, secure builds.
3.  Injects security gates into the `.github/workflows/ci.yml`.

This ensures that every new project starts with a professional security posture without manual configuration.
