use octocrab::models::webhook_events::{
    payload::{PullRequestWebhookEventAction, PullRequestWebhookEventPayload},
    WebhookEvent,
};

use crate::{embed_builder::EmbedBuilder, PULL_REQUEST_COLOR};

pub fn make_pull_request_embed(
    event: WebhookEvent,
    specifics: &PullRequestWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("pull request events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Pull request {}: #{} {}",
        repo_name,
        match specifics.action {
            PullRequestWebhookEventAction::Assigned => {
                let assignee = specifics
                    .assignee
                    .as_ref()
                    .expect("pull request assigned events should always have an assignee");
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
                let reviewer = specifics
                    .requested_reviewer
                    .as_ref()
                    .expect("pull request review requested events should always have a reviewer");
                format!("review requested from {}", reviewer.login)
            }
            _ => {
                return None;
            }
        },
        specifics.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .expect("pull request should always have a title")
    ));

    embed.url(
        specifics
            .pull_request
            .html_url
            .as_ref()
            .expect("pull request should always have an html url")
            .as_str(),
    );

    if matches!(specifics.action, PullRequestWebhookEventAction::Opened) {
        if let Some(ref body) = specifics.pull_request.body {
            embed.description(body);
        }
    }

    embed.color(PULL_REQUEST_COLOR);

    Some(embed)
}

#[cfg(test)]
mod tests {
    use crate::{
        make_embed,
        tests::{embed_context, TestConfig},
    };

    #[test]
    fn opened() {
        let payload = include_str!("../../fixtures/pull_request/opened.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("pull_request", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn opened_by_bot() {
        let payload = include_str!("../../fixtures/pull_request/opened_by_bot.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("pull_request", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn closed() {
        let payload = include_str!("../../fixtures/pull_request/closed.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("pull_request", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn reopened() {
        let payload = include_str!("../../fixtures/pull_request/reopened.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("pull_request", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
