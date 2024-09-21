# Hardcoding all the args for now, can be dynamic later

_default:
  @just --list

# Start a server on http://localhost:3000 which receives webhook events and dumps the payloads into the fixtures folder
listen: 
  python3 server.py

# Requires the following commands to be run first:
#   - gh extension install cli/gh-webhook
#   - gh auth refresh -h github.com -s admin:org_hook
#
# Create a development GitHub webhook and forward all webhook events to http://localhost:3000
register_webhook:
  gh webhook forward --events='*' --org=catppuccin-rfc --url="http://localhost:3000"

# Create a new issue, close and reopen it in catppuccin-rfc/polybar
issues:
  #!/usr/bin/env bash
  ISSUE_URL=$(gh issue create --title "rockdove-{{datetime_utc("%Y%m%d_%H%M%S")}}" --body "rockdove" --repo catppuccin-rfc/polybar)
  gh issue close "$ISSUE_URL"
  gh issue reopen "$ISSUE_URL"

