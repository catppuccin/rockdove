use octocrab::models::webhook_events::{payload::CommitCommentWebhookEventPayload, WebhookEvent};

use crate::{colors::COMMIT_COLOR, embed_builder::EmbedBuilder};

pub fn make_commit_comment_embed(
    event: WebhookEvent,
    specifics: &CommitCommentWebhookEventPayload,
) -> EmbedBuilder {
    let repo = event
        .repository
        .expect("commit comment events should always have a repository");

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
            .expect("commit comment should always have a body")
            .as_str(),
    );
    embed.color(COMMIT_COLOR);

    embed
}

#[cfg(test)]
mod tests {
    use crate::{
        make_embed,
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
