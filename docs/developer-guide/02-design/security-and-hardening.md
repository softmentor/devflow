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
| **Code** | GitHub Advanced Security (CodeQL) | SAST, Secret Leaks, Logic Flaws | Every Push/PR |
| **Dependencies** | Trivy (FS mode) | Known CVEs in OSS libraries | Local CI & Every PR |
| **Images** | Trivy (Image mode) | OS vulnerabilities, Malware | Image Build & Nightly |
| **Infrastructure** | Trivy (Config mode) | Dockerfile/IAC misconfigurations | Local CI & Release |

### Dependency Scanning (Push-Left)
To catch vulnerable libraries before they reach CI, Devflow promotes local dependency scanning. 
- **Local**: `dwf check:security` runs a local filesystem scan.
- **IDE**: We recommend using the **Trivy** or **Snyk** IDE extensions for real-time feedback in VS Code/IntelliJ.

### Image Scanning (Operational Efficiency)
Scanning every image on every PR check can be resource-intensive. Devflow optimizes this by:
- **Build-time**: Scanning occurs only when a new image is generated (e.g., changes to `Dockerfile` or toolchain manifests).
- **Nightly/Release**: A full image scan is performed nightly on the `main` branch and for every tagged release to catch newly discovered CVEs in persistent base images.

## 2. Hardened-by-Default Infrastructure

Every container generated or managed by Devflow is hardened using 2025 industry standards:

### Non-Root Execution
All Devflow-certified images (Rust, Node, Python, Tauri) execute as `dwfuser` (UID 1001). This prevents privilege escalation attacks where a compromised build process could gain root access to the host runner or developer machine.

### Process Hygiene (Tini)
We use `tini` as the container `ENTRYPOINT`. It acts as an init process, correctly forwarding signals (SIGTERM, SIGINT) and reaping zombie processes. This adds stability to long-running CI parallel tasks.

### Rootless Workflow Compatibility
The Devflow executor is tested for compatibility with **rootless Podman** and **Docker Rootless Mode**, ensuring developers don't need `sudo` to run their local verification suites.

## 3. Least Privilege GITHUB_TOKEN

Devflow's `ci:generate` command explicitly configures the GitHub Actions workflow with a root-level `permissions` block:

```yaml
permissions:
  contents: read
```

This prevents the `GITHUB_TOKEN` from having broad write access by default, protecting your repository from unauthorized modifications during the build phase.

## 4. Setup and Automation

When running `dwf init`, Devflow automatically:
1.  Generates the hardened `Dockerfile.devflow`.
2.  Configures `docker-bake.hcl` for high-speed, secure builds.
3.  Injects security gates into the `.github/workflows/ci.yml`.

This ensures that every new project starts with a professional security posture without manual configuration.
