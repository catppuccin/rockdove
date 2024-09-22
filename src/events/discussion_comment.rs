use octocrab::models::webhook_events::{
    payload::{DiscussionCommentWebhookEventAction, DiscussionCommentWebhookEventPayload},
    WebhookEvent,
};

use crate::{colors::DISCUSSION_COLOR, embed_builder::EmbedBuilder};

// TODO: Create a PR to upstream (octocrab) to add typed events so that we don't
// need to use `.get()`, `.as_str()`, etc.

pub fn make_discussion_comment_embed(
    event: WebhookEvent,
    specifics: &DiscussionCommentWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(
        specifics.action,
        DiscussionCommentWebhookEventAction::Created
    ) {
        return None;
    }

    let repo = event
        .repository
        .expect("discussion comment events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] New comment on discussion #{}: {}",
        repo_name,
        specifics
            .discussion
            .get("number")
            .expect("discussion comment should always have a number"),
        specifics
            .discussion
            .get("title")
            .expect("discussion should always have a title")
            .as_str()
            .expect("discussion title should always be a string"),
    ));

    embed.url(
        specifics
            .comment
            .get("html_url")
            .expect("discussion should always have an html url")
            .as_str()
            .expect("discussion html url should always be a string"),
    );

    embed.description(
        specifics
            .comment
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
