use octocrab::models::webhook_events::{
    payload::{DiscussionCommentWebhookEventAction, DiscussionCommentWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::DISCUSSION_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

// TODO: Create a PR to upstream (octocrab) to add typed events so that we don't
// need to use `.get()`, `.as_str()`, etc.

pub fn make_embed(
    event: WebhookEvent,
    specifics: &DiscussionCommentWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    if !matches!(
        specifics.action,
        DiscussionCommentWebhookEventAction::Created
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
        "[{}] New comment on discussion #{}: {}",
        repo_name,
        specifics
            .discussion
            .get("number")
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "discussion.number",
            })?,
        specifics
            .discussion
            .get("title")
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "discussion.title",
            })?
            .as_str()
            .ok_or_else(|| RockdoveError::InvalidField {
                event_type: event.kind.clone(),
                field: "discussion.title",
            })?,
    ));

    embed.url(
        specifics
            .comment
            .get("html_url")
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "comment.html_url",
            })?
            .as_str()
            .ok_or_else(|| RockdoveError::InvalidField {
                event_type: event.kind.clone(),
                field: "comment.html_url",
            })?,
    );

    embed.description(
        specifics
            .comment
            .get("body")
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "comment.body",
            })?
            .as_str()
            .ok_or_else(|| RockdoveError::InvalidField {
                event_type: event.kind.clone(),
                field: "comment.body",
            })?,
    );

    embed.color(DISCUSSION_COLOR);

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
        let payload = include_str!("../../fixtures/discussion_comment/created.json");
        let TestConfig {
            webhook_event: event,
            mut settings,
        } = TestConfig::new("discussion_comment", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
