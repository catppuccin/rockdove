use octocrab::models::webhook_events::{
    payload::{PullRequestWebhookEventAction, PullRequestWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::PULL_REQUEST_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &PullRequestWebhookEventPayload,
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
        "[{}] Pull request {}: #{} {}",
        repo_name,
        match specifics.action {
            PullRequestWebhookEventAction::Assigned => {
                let assignee =
                    specifics
                        .assignee
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "assignee",
                        })?;
                format!("assigned to {}", assignee.login)
            }
            PullRequestWebhookEventAction::Closed => {
                if specifics.pull_request.merged_at.is_some() {
                    "merged".to_string()
                } else {
                    "closed".to_string()
                }
            }
            PullRequestWebhookEventAction::Locked => "locked".to_string(),
            PullRequestWebhookEventAction::Opened => "opened".to_string(),
            PullRequestWebhookEventAction::ReadyForReview => "ready for review".to_string(),
            PullRequestWebhookEventAction::Reopened => "reopened".to_string(),
            PullRequestWebhookEventAction::ReviewRequested => {
                let reviewer = specifics.requested_reviewer.as_ref().ok_or_else(|| {
                    RockdoveError::MissingField {
                        event_type: event.kind.clone(),
                        field: "requested_reviewer",
                    }
                })?;
                format!("review requested from {}", reviewer.login)
            }
            _ => {
                return Ok(None);
            }
        },
        specifics.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "pull_request.title",
            })?
    ));

    embed.url(
        specifics
            .pull_request
            .html_url
            .as_ref()
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "pull_request.html_url",
            })?
            .as_str(),
    );

    if matches!(specifics.action, PullRequestWebhookEventAction::Opened) {
        if let Some(ref body) = specifics.pull_request.body {
            embed.description(body);
        }
    }

    embed.color(PULL_REQUEST_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::{
        events::make_embed,
        tests::{embed_context, TestConfig},
    };
    use std::fs;
    use yare::parameterized;

    #[parameterized(
      opened = { "opened" },
      opened_by_bot = { "opened_by_bot" },
      closed = { "closed" },
      reopened = { "reopened" }
    )]
    fn snapshot(event_type: &str) {
        let event = "pull_request";
        let root = env!("CARGO_MANIFEST_DIR");
        let filename = format!("{root}/fixtures/{event}/{event_type}.json");
        let payload = fs::read_to_string(&filename).expect("fixture exists");
        let TestConfig {
            webhook_event,
            mut settings,
        } = TestConfig::new(event, &payload);

        let embed = make_embed(webhook_event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
