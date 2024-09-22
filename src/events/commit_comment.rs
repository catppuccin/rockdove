use octocrab::models::webhook_events::{payload::CommitCommentWebhookEventPayload, WebhookEvent};

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
    use crate::{
        events::make_embed,
        tests::{embed_context, TestConfig},
    };

    #[test]
    fn created() {
        let payload = include_str!("../../fixtures/commit_comment/created.json");
        let TestConfig {
            webhook_event: event,
            mut settings,
        } = TestConfig::new("commit_comment", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
