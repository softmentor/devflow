---
title: GitHub Repository Standardization
label: devflow.developer-guide.infrastructure.repository
---

# GitHub Repository Standardization

Devflow uses Infrastructure-as-Code (IaC) to standardize repository settings, branch protection, and community templates across the project. This ensures that the repository remains secure, auditable, and follows modern open-source standards.

## Terraform Setup

The repository settings are managed via Terraform in the `.github/settings/terraform` directory.

### Core Files

- **[provider.tf](file://.github/settings/terraform/provider.tf)**: Configures the GitHub provider. Requires a `GITHUB_TOKEN` with administrative access to the repository.
- **[repository.tf](file://.github/settings/terraform/repository.tf)**: Defines the repository configuration, including:
    - Description and visibility.
    - Features (Issues, Projects, Wikis).
    - **Branch Protection**: Enforced rules for the `main` branch:
        - Required pull request reviews.
        - Required status checks (CI must pass).
        - Enforced linear history.
        - Signed commits requirement.
    - **Labels**: Standardized labels for issues and pull requests.

## Usage

Standardization tasks are integrated into the root `Makefile`.

### Preview Changes

To preview what changes Terraform would make to your repository:

```bash
make gh-setup
```

This runs `terraform init` and `terraform plan`.

### Apply Changes

To apply the standardized settings (requires authentication):

```bash
cd .github/settings/terraform
terraform apply
```

> [!IMPORTANT]
> Ensure you have exported a valid `GITHUB_TOKEN` before running Terraform commands.

## Community Templates

Devflow also standardizes the community experience through templates in `.github/`:

- **Issue Templates**: Structured forms for [Bug Reports](file://.github/ISSUE_TEMPLATE/bug_report.yml), [Feature Requests](file://.github/ISSUE_TEMPLATE/feature_request.yml), and [Security Incidents](file://.github/ISSUE_TEMPLATE/security-incident.yml).
- **Pull Request Template**: A [checklist-based template](file://.github/PULL_REQUEST_TEMPLATE.md) for all contributors.
- **Policies**: Standardized [CONTRIBUTING.md](file://CONTRIBUTING.md), [CODE_OF_CONDUCT.md](file://CODE_OF_CONDUCT.md), and [SECURITY.md](file://SECURITY.md).

## Benefits

- **Consistency**: All Devflow-managed repositories follow the same security and governance baseline.
- **Auditability**: Changes to repository settings are reviewed via Pull Requests.
- **Reproducibility**: New repository mirrors can be stood up with identical settings in minutes.
