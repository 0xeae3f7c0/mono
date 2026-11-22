# Repository settings for the monorepo, kept as code: the main branch ruleset
# that makes auto-merge wait for CI. Apply by hand (see README.md); CI only
# formats and validates this configuration.

provider "github" {
  owner = var.owner
}

resource "github_repository_ruleset" "main" {
  name        = "main"
  repository  = var.repository
  target      = "branch"
  enforcement = "active"

  conditions {
    ref_name {
      include = ["~DEFAULT_BRANCH"]
      exclude = []
    }
  }

  # Repository admins (the owner, and OWNER_TOKEN pushes such as auto-format)
  # may bypass the rules.
  bypass_actors {
    actor_id    = 5
    actor_type  = "RepositoryRole"
    bypass_mode = "always"
  }

  rules {
    # Auto-merge only waits for required checks; without these it merges a
    # pull request as soon as auto-merge is enabled.
    required_status_checks {
      strict_required_status_checks_policy = false

      dynamic "required_check" {
        for_each = var.required_checks
        content {
          context = required_check.value
        }
      }
    }
  }
}
