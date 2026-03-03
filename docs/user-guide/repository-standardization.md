---
title: Repository Standardization Setup
label: devflow.user-guide.repository-standardization
---

# Repository Standardization Setup

Devflow provides a standardized way to set up your GitHub repository with recommended best practices for security, governance, and community engagement. This is achieved using Infrastructure-as-Code (Terraform) templates.

## Prerequisites

Before setting up your repository standardization:

1.  **Terraform**: Ensure you have Terraform installed. You can install it via:
    ```bash
    make setup-tools
    ```
2.  **GitHub Token**: You need a Personal Access Token (PAT) with `repo` and `admin:org` permissions (if managing an organization repo).
    ```bash
    export GITHUB_TOKEN=your_token_here
    ```

## Step 1: Initialize Templates

When you run `dwf init`, Devflow populates the `.github/settings/terraform` directory with baseline templates.

- `provider.tf`: Configures the GitHub provider connection.
- `repository.tf`: Defines branch protection, labels, and repository features.

### Importing Existing Repository

If you are standardizing an existing repository, you may need to import it into Terraform state before applying changes:

```bash
cd .github/settings/terraform
terraform init
terraform import github_repository.devflow devflow
```

## Step 2: Preview the Configuration

Use the built-in `make` command to preview what changes will be applied to your repository:

```bash
make gh-setup
```

This command will:
1.  Initialize the Terraform environment.
2.  Run a `terraform plan` to show the difference between your current repository settings and the Devflow standard.

You can also run this manually:

```bash
terraform plan
```

## Step 3: Apply the Standardization

If you are satisfied with the preview, you can apply the settings:

```bash
terraform apply
```

This will enforce:
- **Branch Protection**: Requires status checks (CI) and PR reviews before merging to `main`.
- **Labels**: Standardizes issue labels for better project management.
- **Security**: Enables automated vulnerability alerts and secret scanning.

## Community & Governance

Standardizing your repository also includes deploying shared community files:

- **Issue Templates**: Found in `.github/ISSUE_TEMPLATE/` (Bug reports, Feature requests).
- **PR Template**: Found in `.github/PULL_REQUEST_TEMPLATE.md`.
- **Policies**: Standard `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md` files are placed in your repository root.

## Troubleshooting

- **404 Not Found**: Ensure your `GITHUB_TOKEN` has the correct permissions and the repository URL in your git config is correct.
- **State Lock**: If a previous run crashed, you may need to run `terraform force-unlock`.

For more technical details on the Terraform implementation, see the [Developer Guide](file:///Users/jinythattil/jt/code/softmentor/devflow/docs/developer-guide/04-infrastructure/repository-standardization.md).
