use octocrab::models::webhook_events::{
    WebhookEvent,
    payload::{
        PullRequestReviewCommentWebhookEventAction, PullRequestReviewCommentWebhookEventPayload,
    },
};

use crate::{
    colors::PULL_REQUEST_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &PullRequestReviewCommentWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    if !matches!(
        specifics.action,
        PullRequestReviewCommentWebhookEventAction::Created
    ) {
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

    embed.title(&format!(
        "[{}] New review comment on pull request #{}: {}",
        repo_name,
        specifics.pull_request.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "pull_request.title",
            })?,
    ));
    embed.url(specifics.comment.html_url.as_str());
    embed.description(specifics.comment.body.as_str());
    embed.color(PULL_REQUEST_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    #[test]
    fn created() {
        snapshot_test!("pull_request_review_comment", "created");
    }
}
