use octocrab::models::webhook_events::{
    payload::{DiscussionWebhookEventAction, DiscussionWebhookEventPayload},
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
    specifics: &DiscussionWebhookEventPayload,
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
        "[{}] Discussion {}: #{} {}",
        repo_name,
        match specifics.action {
            DiscussionWebhookEventAction::Created => "created".to_string(),
            DiscussionWebhookEventAction::Closed => "closed".to_string(),
            DiscussionWebhookEventAction::Reopened => "reopened".to_string(),
            _ => {
                return Ok(None);
            }
        },
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
            .discussion
            .get("html_url")
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "discussion.html_url",
            })?
            .as_str()
            .ok_or_else(|| RockdoveError::InvalidField {
                event_type: event.kind.clone(),
                field: "discussion.html_url",
            })?,
    );

    if matches!(specifics.action, DiscussionWebhookEventAction::Created) {
        embed.description(
            specifics
                .discussion
                .get("body")
                .ok_or_else(|| RockdoveError::MissingField {
                    event_type: event.kind.clone(),
                    field: "discussion.body",
                })?
                .as_str()
                .ok_or_else(|| RockdoveError::InvalidField {
                    event_type: event.kind.clone(),
                    field: "discussion.body",
                })?,
        );
    }

    embed.color(DISCUSSION_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    use yare::parameterized;

    #[parameterized(
        created = { "created" },
        closed = { "closed" },
        reopened = { "reopened" }
      )]
    fn snapshot(event_type: &str) {
        snapshot_test!("discussion", event_type);
    }
}
