use octocrab::models::webhook_events::{WebhookEvent, payload::CommitCommentWebhookEventPayload};

use crate::{
    colors::COMMIT_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &CommitCommentWebhookEventPayload,
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
        "[{}] New comment on commit {}",
        repo_name, specifics.comment.commit_id,
    ));
    embed.url(specifics.comment.html_url.as_str());
    embed.description(
        specifics
            .comment
            .body
            .as_ref()
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "comment.body",
            })?
            .as_str(),
    );
    embed.color(COMMIT_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    #[test]
    fn created() {
        snapshot_test!("commit_comment", "created");
    }
}
