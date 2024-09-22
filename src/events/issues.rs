use octocrab::models::webhook_events::{
    payload::{IssuesWebhookEventAction, IssuesWebhookEventPayload},
    WebhookEvent,
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
            IssuesWebhookEventAction::Closed => "closed".to_string(),
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
    use crate::{
        events::make_embed,
        tests::{embed_context, TestConfig},
    };
    use std::fs;
    use yare::parameterized;

    #[parameterized(
        opened = { "opened" },
        closed = { "closed" },
        reopened = { "reopened" }
      )]
    fn snapshot(event_type: &str) {
        let event = "issues";
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
