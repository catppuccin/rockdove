use octocrab::models::webhook_events::{
    payload::{DiscussionWebhookEventAction, DiscussionWebhookEventPayload},
    WebhookEvent,
};

use crate::{embed_builder::EmbedBuilder, DISCUSSION_COLOR};

pub fn make_discussion_embed(
    event: WebhookEvent,
    specifics: &DiscussionWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("discussion events should always have a repository");

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
                return None;
            }
        },
        specifics
            .discussion
            .get("number")
            .expect("discussion should always have a number"),
        specifics
            .discussion
            .get("title")
            .expect("discussion should always have a title")
            .as_str()
            .expect("discussion title should always be a string"),
    ));

    embed.url(
        specifics
            .discussion
            .get("html_url")
            .expect("discussion should always have an html url")
            .as_str()
            .expect("discussion html url should always be a string"),
    );

    embed.description(
        specifics
            .discussion
            .get("body")
            .expect("discussion should always have a body")
            .as_str()
            .expect("discussion body should always be a string"),
    );

    embed.color(DISCUSSION_COLOR);

    Some(embed)
}

#[cfg(test)]
mod tests {
    use crate::{
        make_embed,
        tests::{embed_context, TestConfig},
    };

    #[test]
    fn created() {
        let payload = include_str!("../../fixtures/discussion/created.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("discussion", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn closed() {
        let payload = include_str!("../../fixtures/discussion/closed.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("discussion", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn reopened() {
        let payload = include_str!("../../fixtures/discussion/reopened.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("discussion", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
