use octocrab::models::webhook_events::{
    payload::{IssueCommentWebhookEventAction, IssueCommentWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::{ISSUE_COLOR, PULL_REQUEST_COLOR},
    embed_builder::EmbedBuilder,
};

pub fn make_issue_comment_embed(
    event: WebhookEvent,
    specifics: &IssueCommentWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(specifics.action, IssueCommentWebhookEventAction::Created) {
        return None;
    }

    let repo = event
        .repository
        .expect("issue comment events should always have a repository");

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
        created = { "created" },
        created_on_pull_request = { "created_on_pull_request" },
      )]
    fn snapshot(event_type: &str) {
        let event = "issue_comment";
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
