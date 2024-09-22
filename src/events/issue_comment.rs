use octocrab::models::webhook_events::{
    payload::{IssueCommentWebhookEventAction, IssueCommentWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::{ISSUE_COLOR, PULL_REQUEST_COLOR},
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &IssueCommentWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    if !matches!(specifics.action, IssueCommentWebhookEventAction::Created) {
        return Ok(None);
    }

    let repo = event
        .repository
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "repository",
        })?;

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);
    let target = if specifics.issue.pull_request.is_some() {
        "pull request"
    } else {
        "issue"
    };

    embed.title(&format!(
        "[{}] New comment on {} #{}: {}",
        repo_name, target, specifics.issue.number, specifics.issue.title,
    ));

    embed.url(specifics.comment.html_url.as_str());

    if let Some(ref body) = specifics.comment.body {
        embed.description(body);
    }

    embed.color(if specifics.issue.pull_request.is_some() {
        PULL_REQUEST_COLOR
    } else {
        ISSUE_COLOR
    });

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    use yare::parameterized;

    #[parameterized(
        created = { "created" },
        created_on_pull_request = { "created_on_pull_request" },
      )]
    fn snapshot(event_type: &str) {
        snapshot_test!("issue_comment", event_type);
    }
}
