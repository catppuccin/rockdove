use octocrab::models::{
    pulls::ReviewState,
    webhook_events::{
        payload::{PullRequestReviewWebhookEventAction, PullRequestReviewWebhookEventPayload},
        WebhookEvent,
    },
};

use crate::{colors::PULL_REQUEST_COLOR, embed_builder::EmbedBuilder};

pub fn make_pull_request_review_embed(
    event: WebhookEvent,
    specifics: &PullRequestReviewWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(
        specifics.action,
        PullRequestReviewWebhookEventAction::Submitted
    ) {
        return None;
    }

    let repo = event
        .repository
        .expect("pull request review events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Pull request {}: #{} {}",
        repo_name,
        match specifics
            .review
            .state
            .expect("pull request review should always have a state")
        {
            ReviewState::Approved => "approved",
            ReviewState::ChangesRequested => "changes requested",
            ReviewState::Commented => "reviewed",
            _ => return None,
        },
        specifics.pull_request.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .expect("pull request should always have a title"),
    ));

    embed.url(specifics.review.html_url.as_str());

    if let Some(ref body) = specifics.review.body {
        embed.description(body);
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
    use std::fs;
    use yare::parameterized;

    #[parameterized(
        approved = { "approved" },
        changes_requested = { "changes_requested" },
        commented = { "commented" }
      )]
    fn snapshot(event_type: &str) {
        let event = "pull_request_review";
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
