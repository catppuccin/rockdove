use octocrab::models::{
    issues::IssueStateReason,
    webhook_events::{
        WebhookEvent,
        payload::{IssuesWebhookEventAction, IssuesWebhookEventPayload},
    },
};

use crate::{
    colors::ISSUE_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &IssuesWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    let repo = event
        .repository
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "repository",
        })?;

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Issue {}: #{} {}",
        repo_name,
        match specifics.action {
            IssuesWebhookEventAction::Assigned => {
                let assignee = specifics.issue.assignee.as_ref().ok_or_else(|| {
                    RockdoveError::MissingField {
                        event_type: event.kind.clone(),
                        field: "issue.assignee",
                    }
                })?;
                format!("assigned to {}", assignee.login)
            }
            IssuesWebhookEventAction::Closed => {
                specifics.issue.state_reason.as_ref().map_or_else(
                    || "closed".to_string(),
                    |state_reason| match state_reason {
                        IssueStateReason::NotPlanned => "closed as not planned".to_string(),
                        IssueStateReason::Reopened => "reopened".to_string(),
                        IssueStateReason::Duplicate => "closed as duplicate".to_string(),
                        _ => "closed".to_string(),
                    },
                )
            }
            IssuesWebhookEventAction::Locked => "locked".to_string(),
            IssuesWebhookEventAction::Opened => "opened".to_string(),
            IssuesWebhookEventAction::Pinned => "pinned".to_string(),
            IssuesWebhookEventAction::Reopened => "reopened".to_string(),
            // TODO: this would be nice
            // IssuesWebhookEventAction::Transferred => {
            //     todo!()
            // }
            _ => {
                return Ok(None);
            }
        },
        specifics.issue.number,
        specifics.issue.title,
    ));

    embed.url(specifics.issue.html_url.as_str());

    if matches!(specifics.action, IssuesWebhookEventAction::Opened) {
        if let Some(ref body) = specifics.issue.body {
            embed.description(body);
        }
    }

    embed.color(ISSUE_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    use yare::parameterized;

    #[parameterized(
        opened = { "opened" },
        closed = { "closed" },
        closed_as_duplicate = { "closed_as_duplicate" },
        closed_as_not_planned = { "closed_as_not_planned" },
        reopened = { "reopened" }
      )]
    fn snapshot(event_type: &str) {
        snapshot_test!("issues", event_type);
    }
}
