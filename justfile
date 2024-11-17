# Hardcoding all the args for now, can be dynamic later

_default:
  @just --list

# Start a server on http://localhost:3000 which receives webhook events and dumps the payloads into the fixtures folder
listen: 
  python3 server.py

# Create a new issue, close and reopen it in catppuccin-rfc/polybar
issues:
  #!/usr/bin/env bash
  ISSUE_URL=$(gh issue create --title "rockdove-{{datetime_utc("%Y%m%d_%H%M%S")}}" --body "rockdove" --repo catppuccin-rfc/polybar)
  gh issue close "$ISSUE_URL"
  gh issue reopen "$ISSUE_URL"

# Rename an existing repository under catppuccin-rfc
repository_rename current_repo new_repo:
  gh repo rename {{new_repo}} --repo catppuccin-rfc/{{current_repo}}

# The reason for using `gh api`: https://github.com/cli/cli/issues/5292
#
# Transfer an existing repository to a new owner
repository_transfer current_owner_plus_repo new_owner:
  gh api repos/{{current_owner_plus_repo}}/transfer -f new_owner={{new_owner}}

# Create a pull request, close and reopen it in catppuccin-rfc/polybar
pull_request:
  #!/usr/bin/env bash
  PR_URL=$(gh pr create --base main --head sgoudham-patch-1 --title "rockdove-{{datetime_utc("%Y%m%d_%H%M%S")}}" --body "rockdove" --repo catppuccin-rfc/polybar)
  gh pr close "$PR_URL"
  gh pr reopen "$PR_URL"