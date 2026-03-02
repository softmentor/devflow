# 1. Define the Repository & General Settings
resource "github_repository" "devflow" {
  name        = "devflow"
  description = "Modern developer workflow automation"
  visibility  = "public"

  # Feature Flags
  has_issues      = true
  has_projects    = false
  has_wiki        = false
  has_discussions = false # Set to true if you want to enable GitHub Discussions

  # Merge Strategies
  allow_merge_commit = true
  allow_squash_merge = true
  allow_rebase_merge = false
  delete_branch_on_merge = true

  # Security Settings: Dependabot
  vulnerability_alerts = true 

  security_and_analysis {
    secret_scanning {
      status = "enabled"
    }
    secret_scanning_push_protection {
      status = "enabled"
    }
  }
}

# 2. Define a Branch Ruleset (Modern Branch Protection)
resource "github_repository_ruleset" "main_protection" {
  name        = "Production Standards"
  repository  = github_repository.devflow.name
  target      = "branch"
  enforcement = "active"

  conditions {
    ref_name {
      include = ["~DEFAULT_BRANCH"]
      exclude = []
    }
  }

  rules {
    deletion = true
    non_fast_forward = true

    pull_request {
      required_approving_review_count = 0 # Adjust as needed
      dismiss_stale_reviews_on_push   = true
      require_code_owner_reviews      = false
      require_last_push_approval      = true
      required_review_thread_resolution = true
    }

    required_status_checks {
      strict_required_status_checks_policy = true
      
      required_check {
        context = "fmt-check"
      }
      required_check {
        context = "lint-static"
      }
      required_check {
        context = "build-debug"
      }
      required_check {
        context = "test-unit"
      }
      required_check {
        context = "test-integration"
      }
    }
    
    required_signatures = false # Set to true if GPG signing is mandatory
  }
}
